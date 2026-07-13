# Parsing PostgreSQL Packets in Rust with `packet_parser`

PostgreSQL traffic is a good example of why packet parsing needs to be more than header decoding.
The protocol has a startup phase, typed frontend/backend messages, length-prefixed fields, and enough structure to make both strict validation and partial recovery useful.

In `packet_parser`, PostgreSQL support was added as a native application-layer protocol parser.
The parser detects traffic on the standard PostgreSQL port, decodes startup and typed messages, and exposes typed errors when the packet is malformed.

Repository: https://github.com/Akmot9/Packet-parser

## Why PostgreSQL Is Interesting To Parse

At first glance, PostgreSQL packets look simple:

- one startup exchange without a message type byte
- then typed messages with a leading tag byte
- length fields that define how much payload follows
- a mix of strings, binary fields, and backend responses

In practice, that means a parser needs to be careful about:

- startup packets versus regular messages
- length field interpretation
- NUL-terminated strings
- trailing bytes that should not be silently ignored
- malformed packets that are still useful for diagnostics

That combination makes PostgreSQL a good fit for a typed Rust parser.

## The Parsing Model

`packet_parser` keeps the parsing flow layered:

- Ethernet
- IP
- TCP or UDP
- application protocol

PostgreSQL is recognized at the application layer when the transport port matches the standard PostgreSQL port.
Once detected, the parser interprets the payload as PostgreSQL protocol data instead of leaving it as raw bytes.

The main goal is not only to decode valid messages.
It is also to preserve as much structure as possible when the packet is incomplete or malformed.

## Packet Structure

The PostgreSQL frontend/backend protocol has two major shapes.

The first is the startup message:

- no initial message type byte
- a protocol version or startup code
- key/value parameters terminated by a double NUL

The second is the typed message family:

- a one-byte message type
- a length field
- a message body whose shape depends on the type

That distinction matters because the parser cannot treat all messages the same way.
Startup packets and regular typed packets are structurally different.

## Typed Errors

The PostgreSQL parser uses dedicated error values instead of generic failures.
That makes debugging and testing much more useful.

Examples of reported issues include:

- empty packet
- buffer too small
- invalid message type
- invalid message length
- length mismatch
- invalid UTF-8 in a string field
- missing NUL terminator
- unsupported startup code

This is a deliberate choice.
Network traffic is messy, and a parser should explain what went wrong instead of collapsing everything into a single `false` or `None`.

## Zero-Copy Where It Helps

The parser borrows payload slices when possible.
That avoids copying large portions of the packet and keeps the parsed data linked to the original buffer.

This is especially useful for:

- raw payloads
- string fields
- message bodies that are only inspected, not mutated

Rust lifetimes make that relationship explicit, which is a good match for packet data that should not outlive the source buffer.

## Example Usage

You can inspect a full packet first and then decode the PostgreSQL payload:

```rust
use hex::decode;
use packet_parser::parse::application::protocols::postgresql::PostgreSqlPacket;
use packet_parser::PacketFlow;
use std::convert::TryFrom;

fn main() {
    let packet = decode("...hex payload...").expect("hex decoding failed");

    let flow = PacketFlow::try_from(packet.as_slice()).expect("invalid frame");

    if let Some(transport) = &flow.transport {
        if let Some(payload) = transport.payload {
            let pg = PostgreSqlPacket::try_from(payload).expect("invalid PostgreSQL packet");
            println!("{pg:#?}");
        }
    }
}
```

If you only want to test the PostgreSQL parser directly, you can feed it the application payload without building the full flow first.

## What The Parser Checks

The implementation is strict about structure.

It verifies, among other things:

- message length consistency
- field length validity
- packet truncation
- UTF-8 correctness where text is expected
- proper termination of string fields
- valid startup layout

That strictness is important because PostgreSQL packets often appear in captures alongside other TCP traffic.
Without enough validation, a parser can easily misclassify arbitrary bytes as a valid message.

## Tests That Matter

The PostgreSQL support is covered at two levels:

- protocol-level tests for the PostgreSQL parser itself
- packet-level tests for `PacketFlow` detection on a full captured frame

That split is useful because it verifies both the parser and the detection logic.

The packet-level tests also prove an important point: detection does not depend on guessing from raw bytes alone.
The parser can recognize PostgreSQL on the standard port and then decode the payload into typed structures.

## Why This Fits `packet_parser`

PostgreSQL support follows the same design rules as the rest of the crate:

- parse layer by layer
- keep protocol modules separate
- prefer typed data over ad hoc blobs
- return precise errors
- keep the crate easy to extend

That makes the PostgreSQL module more than an isolated feature.
It fits the broader parsing model the crate already uses for DNS, HTTP, TLS, MQTT, Modbus/TCP, S7Comm, SNMP, EtherNet/IP, and other protocols.

## Closing Note

PostgreSQL is not the simplest protocol to parse, but that is exactly what makes it useful as a crate feature.
It exercises message framing, typed decoding, string handling, and best-effort detection in a way that mirrors real-world captures.

For network analysis tools written in Rust, that is the sort of protocol support that tends to matter in practice.
