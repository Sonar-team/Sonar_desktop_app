# Recursive Tunnel Decapsulation in `packet_parser`: Why `PacketFlow` Changed

Most packet parsers start with a simple mental model:

```text
one captured frame -> one parsed flow
```

That works for ordinary Ethernet, IP, TCP, UDP, and application traffic.
It starts to break down when the packet carries another packet inside its payload.

That is what tunnels do.

With the recursive tunnel decapsulation work in `packet_parser`, a single wire packet can now expose several flow levels:

```text
outer Ethernet/IP/UDP/CAPWAP
    inner 802.11/LLC/SNAP/IP/TCP/application
```

This required a public API change.
`PacketFlow` now has an `inner` field, and the crate version moved to `3.0.0` because code that builds `PacketFlow` with struct literals must be updated.

Repository: https://github.com/Akmot9/Packet-parser

## The Old Model

Before this change, `PacketFlow` represented one decoded packet path:

- data link layer
- internet layer
- transport layer
- best-effort application layer

That was enough for direct traffic.
For example, a TCP packet carrying PostgreSQL or a UDP packet carrying SNMP could be represented naturally.

But encapsulated traffic was flattened too early.
If the outer packet was CAPWAP over UDP, the parser could describe the outer Ethernet, IP, and UDP layers, but the real client/server conversation inside the wireless payload stayed hidden as raw bytes.

For traffic analysis, that is a serious limitation.
The useful information is often inside the tunnel, not on the outer transport line.

## The New Model

`PacketFlow` now supports recursive flows:

```rust
pub struct PacketFlow<'a> {
    pub data_link: DataLink<'a>,
    pub internet: Option<Internet<'a>>,
    pub transport: Option<Transport<'a>>,
    pub application: Option<Application>,
    pub inner: Option<Box<PacketFlow<'a>>>,
}
```

The outer `PacketFlow` still describes the packet as it appears on the wire.
If the transport payload contains a supported tunnel, the parser records the tunnel as the outer application protocol and parses the encapsulated packet into `inner`.

For a CAPWAP packet, that means:

- the outer flow keeps Ethernet, IPv4, UDP, and application `"CAPWAP"`
- the inner flow contains the reconstructed data-link layer from the 802.11 frame
- the inner flow then continues through IPv4 or IPv6, TCP or UDP, and application detection

The result is still one `PacketFlow` root, but it can now contain another `PacketFlow`.

## Why This Is A Breaking Change

Rust struct literals must mention every public field.

Code like this used to compile:

```rust
let flow = PacketFlow {
    data_link,
    internet: None,
    transport: None,
    application: None,
};
```

It now needs an explicit `inner` value:

```rust
let flow = PacketFlow {
    data_link,
    internet: None,
    transport: None,
    application: None,
    inner: None,
};
```

That is the main source-level break.
Most callers that only use `PacketFlow::try_from(frame)` do not need to change anything.
Callers that construct `PacketFlow` manually in tests, fixtures, displays, or examples must add `inner: None`.

This is why the change is a major version bump instead of a minor feature release.

## Why `inner` Is Boxed

A recursive Rust type cannot contain itself directly:

```rust
// impossible: infinite size
pub inner: Option<PacketFlow<'a>>
```

The `Box` gives the recursive field a known size.
The outer flow stores a pointer to the inner flow only when encapsulation is detected.

For normal packets, the field is just:

```rust
inner: None
```

That keeps the common case small while still allowing nested tunnel parsing.

## Reading Tunneled Packets

If you only care about the outer packet, existing access patterns still work:

```rust
let flow = PacketFlow::try_from(frame)?;

if let Some(app) = &flow.application {
    println!("outer application: {}", app.application_protocol);
}
```

For CAPWAP, the outer application protocol is reported as `"CAPWAP"`.
The encapsulated conversation is available through `inner`:

```rust
let flow = PacketFlow::try_from(frame)?;

if let Some(inner) = flow.inner.as_deref() {
    if let Some(transport) = &inner.transport {
        println!("inner transport: {:?}", transport.protocol);
        println!("inner source port: {:?}", transport.source_port);
        println!("inner destination port: {:?}", transport.destination_port);
    }
}
```

That makes the distinction explicit:

- `flow` is what was captured on the wire
- `flow.inner` is what was carried inside the tunnel

## Flattening A Flow

For tools that want to iterate over every visible conversation, `PacketFlow::flatten()` returns the chain from outermost to innermost:

```rust
let flow = PacketFlow::try_from(frame)?;

for level in flow.flatten() {
    if let Some(app) = &level.application {
        println!("application: {}", app.application_protocol);
    }
}
```

A normal packet returns one entry.
A CAPWAP packet with one decoded inner conversation returns two entries:

```text
0: outer Ethernet/IP/UDP/CAPWAP
1: inner 802.11-derived data link/IP/TCP/...
```

This is useful for exporters, logging, tests, and inspection tools that do not want to special-case recursion manually.

## What CAPWAP Decapsulation Does

The first supported recursive tunnel path is:

```text
CAPWAP-Data over UDP 5247
    -> IEEE 802.11 data frame
    -> LLC/SNAP
    -> inner L3 payload
```

The parser handles the practical details that appear in real captures:

- CAPWAP data-plane traffic on UDP port `5247`
- plaintext CAPWAP data packets
- IEEE 802.11 data frames
- ToDS and FromDS address mapping
- WDS frames with Address4
- QoS data headers
- Cisco captures where the 802.11 Frame Control bytes can appear swapped
- LLC/SNAP headers carrying an EtherType

After peeling those headers, the parser rebuilds an internal `DataLink` value.
That lets the existing IPv4, IPv6, TCP, UDP, and application parsers run normally on the inner payload.

The important design point is reuse.
Tunnel support does not create a separate parsing world.
It extracts the inner packet and sends it back through the same layered parser.

## Graceful Degradation

Tunnel detection is deliberately conservative.

If the packet is encrypted, truncated, malformed, or uses a tunnel shape the crate does not support yet, parsing does not fail just because the inner packet could not be decoded.

Instead, `inner` remains `None`, and the outer flow is still returned.

That behavior matters for real captures.
CAPWAP can be protected with DTLS, captures can be cut short, and not every payload uses the LLC/SNAP form needed to recover an EtherType.
The parser should preserve the outer layers even when it cannot safely decode the inner ones.

The recursion also has a depth guard:

```text
MAX_TUNNEL_DEPTH = 4
```

That prevents malformed or adversarial traffic from creating unbounded tunnel recursion.

## Migration Checklist

Most users only need to check a few places.

If you call:

```rust
PacketFlow::try_from(frame)
```

you probably do not need to change your parsing code.

If you build `PacketFlow` manually, add:

```rust
inner: None
```

If your tool exports or displays parsed flows, decide whether it should show only the outer packet or also walk `flow.inner`.
For most analysis tools, `flow.flatten()` is the better default because it exposes the real conversation inside supported tunnels.

If you compare or hash `PacketFlow` values, remember that `inner` is now part of equality and hashing.
Two packets with the same outer layers but different decoded inner flows are not the same parsed result.

## Testing With Real Frames

The change was tested with real CAPWAP frames.

One test covers a ToDS packet with a longer CAPWAP header.
The outer flow is UDP destination port `5247`, reported as `"CAPWAP"`.
The inner flow is IPv4/TCP and exposes the real client-side conversation.

Another test covers the return direction with a shorter CAPWAP header and an IEEE 802.11 FromDS frame.
That verifies the address mapping in the opposite direction and confirms that the inner TCP ports are reversed as expected.

Both tests assert that:

- the outer flow is still visible
- the tunnel is reported as an application protocol on the outer flow
- `inner` contains the decoded inner packet
- `flatten()` returns two flow levels

This is the behavior users usually want from a packet parser: keep the capture context, but do not hide the conversation inside the tunnel.

## Why The API Shape Is Worth It

Adding a public field is disruptive, so it should buy something concrete.

In this case, the old model could not represent nested packets without either dropping information or inventing a side channel.
The new model keeps the existing layered design and extends it recursively.

It also keeps ownership simple:

- parsed fields still borrow from the original packet buffer
- normal packets remain a single flow
- tunneled packets attach their inner flow only when decoding succeeds
- callers can choose between direct access and `flatten()`

That makes the API more honest.
A captured packet is not always just one flow.
Sometimes it is a transport wrapper around another packet, and the parsed data model now reflects that.

## Closing Note

The `PacketFlow.inner` change is small in code but important in meaning.

It moves `packet_parser` from single-level packet inspection toward recursive packet inspection.
That is necessary for real-world network analysis, where tunnels are common and the useful signal often lives behind one or more encapsulation layers.

The migration cost is straightforward: add `inner: None` to manual `PacketFlow` literals and use `flatten()` when you want every decoded level.
In return, `packet_parser` can now expose both the tunnel and the conversation inside it without losing the layered model that made the crate useful in the first place.
