# Packet Parser: A Rust Crate for Layered Network Packet Analysis

Network packets are small, dense, and unforgiving. A few bytes can represent an Ethernet frame, an IPv4 header, a TCP segment, and an industrial protocol message all at once. If you work on network monitoring, packet inspection, cybersecurity tooling, or industrial network analysis, you quickly run into the same problem: raw bytes are easy to capture, but hard to understand safely and consistently.

`packet_parser` is a Rust crate built to make that work more structured.

The goal is straightforward: take a raw network frame and progressively decode it across layers, from Ethernet up to application protocols, while keeping the parser modular, explicit, and easy to extend.

```toml
[dependencies]
packet_parser = "1.5.0"
```

Repository: https://github.com/Akmot9/Packet-parser

## Why Another Packet Parser?

Packet parsing usually sits at the boundary between performance and correctness.

On one side, packet analysis often happens in hot paths: capture pipelines, monitoring agents, offline PCAP processors, intrusion detection experiments, or protocol research tools. On the other side, a parser cannot be casual with malformed data. Network traffic is messy. Captures contain truncated frames, unsupported protocols, malformed headers, vendor-specific behavior, and sometimes intentionally hostile payloads.

`packet_parser` was designed around a few practical ideas:

- Parse packets layer by layer.
- Keep protocol modules independent.
- Return typed errors instead of vague failures.
- Avoid unnecessary copies where borrowed packet slices are enough.
- Make it realistic to add new protocols without rewriting the crate.

This makes the crate useful both as a packet inspection library and as a base for learning how real protocol parsers are organized in Rust.

## A Layered Parsing Model

The main abstraction is `PacketFlow`.

It represents a packet as a progression through common network layers:

- Data link layer
- Internet layer
- Transport layer
- Application layer

Unsupported protocols do not necessarily make the whole parse fail. Instead, the parser keeps as much information as it can. This is important in real captures: if the application protocol is unknown, the Ethernet, IP, and TCP/UDP metadata may still be valuable.

A typical flow looks like this:

```rust
use packet_parser::parse::PacketFlow;

fn inspect_frame(frame: &[u8]) {
    match PacketFlow::try_from(frame) {
        Ok(flow) => {
            println!("{:#?}", flow);
        }
        Err(err) => {
            eprintln!("failed to parse packet: {err}");
        }
    }
}
```

Internally, each layer receives the payload of the previous one. Ethernet exposes the network payload. IPv4 or IPv6 exposes the transport payload. TCP or UDP exposes the application payload. The application layer then attempts best-effort protocol recognition.

That layered approach keeps the parser understandable and makes partial decoding natural.

## Protocol Coverage

The crate already supports several protocol families across the stack.

At the lower layers, it parses Ethernet, VLAN tags, IPv4, IPv6, ARP, TCP, UDP, and Profinet-related network data.

At the application and industrial protocol layer, it includes support for protocols such as:

- DNS
- HTTP
- TLS records
- DHCP and DHCPv6
- NTP
- MQTT
- Bitcoin protocol messages
- Modbus/TCP
- S7Comm
- OPC UA TCP
- AMS
- GIOP
- SRVLOC
- SNMP
- EtherNet/IP encapsulation

The industrial protocol coverage is intentional. Many packet parsing libraries focus mostly on web protocols. `packet_parser` is also interested in operational technology and ICS environments, where protocols like Modbus/TCP, S7Comm, OPC UA, Profinet, SNMP, and EtherNet/IP are common.

## Recent Additions: SNMP and EtherNet/IP

Two recent additions illustrate the direction of the project.

### SNMP

SNMP parsing is implemented with a strict BER/ASN.1 reader. The parser supports SNMP v1, v2c, and v3 message structures, standard PDUs, trap handling, and variable bindings.

The parser keeps variable data as borrowed slices where appropriate, so the original packet remains the source of truth.

The SNMP module also includes tests for valid messages and malformed input, including:

- SNMP v2c `GetRequest`
- SNMP v3 scoped PDU
- invalid top-level ASN.1 tag
- unsupported version
- invalid PDU usage across versions

### EtherNet/IP Encapsulation

EtherNet/IP support focuses on the encapsulation layer.

The parser handles the 24-byte encapsulation header, validates command length, checks command codes, verifies the options field, and parses key command data such as `RegisterSession`, `SendRRData`, and `SendUnitData`.

For `SendRRData` and `SendUnitData`, it also parses the Common Packet Format item list.

One design choice is worth mentioning: EtherNet/IP is detected from the payload structure itself, not from a port number. That makes the parser more flexible when traffic is captured on non-standard ports or in test environments.

To reduce false positives, the EtherNet/IP parser is strict about:

- known encapsulation commands
- exact declared payload length
- valid `RegisterSession` protocol version
- `options == 0`
- valid Common Packet Format structure

This strictness matters because application-layer detection can otherwise collide with more permissive parsers.

## Typed Errors and Separate Checks

Each protocol gets its own error type.

For example, EtherNet/IP parsing can fail with errors such as:

- packet too short
- unknown command
- length mismatch
- invalid options field
- invalid `RegisterSession` payload
- truncated CPF item data

This is more useful than returning a generic `false` or a string. A caller can log precise failures, build metrics around malformed traffic, or decide whether to treat a packet as unsupported or suspicious.

The project also separates validation logic into `checks` modules. That keeps parser files focused on structure and extraction, while validation rules live in a predictable place.

The pattern for adding a protocol is generally:

```text
src/parse/application/protocols/my_protocol.rs
src/checks/application/my_protocol.rs
src/errors/application/my_protocol.rs
```

Then the protocol is wired into the application protocol enum and detection flow.

This layout is simple, but it scales well as the number of supported protocols grows.

## Zero-Copy Where It Makes Sense

Packet parsers often face a tradeoff between convenience and allocation.

Copying every payload into a `Vec<u8>` is easy, but it adds overhead and disconnects parsed fields from the original packet. `packet_parser` prefers borrowed slices for variable-length data when the protocol allows it.

For example, protocol payloads, raw fields, object identifiers, context data, and command data can often be represented as `&[u8]`.

That approach has two advantages:

- It avoids unnecessary allocation in parsing paths.
- It keeps the original packet buffer as the source of truth.

Rust's lifetimes make this explicit. If a parsed structure borrows from the packet, the compiler tracks that relationship.

## Example: Parsing an Ethernet Frame

The crate can be used at different levels. If you only care about Ethernet metadata, you can parse the data link layer directly:

```rust
use packet_parser::parse::data_link::DataLink;

let raw_packet: [u8; 18] = [
    0x2C, 0xFD, 0xA1, 0x3C, 0x4D, 0x5E, // destination MAC
    0x64, 0x6E, 0xE0, 0x12, 0x34, 0x56, // source MAC
    0x08, 0x00,                         // IPv4 ethertype
    0x45, 0x00, 0x00, 0x54,             // payload fragment
];

let datalink = DataLink::try_from(raw_packet.as_ref())
    .expect("valid Ethernet frame");

println!("{datalink:?}");
```

If you want the full layered model, use `PacketFlow`:

```rust
use packet_parser::parse::PacketFlow;

let flow = PacketFlow::try_from(raw_frame.as_slice())?;

if let Some(transport) = &flow.transport {
    println!("transport: {:?}", transport.protocol);
    println!("source port: {:?}", transport.source_port);
    println!("destination port: {:?}", transport.destination_port);
}

if let Some(application) = &flow.application {
    println!("application: {}", application.application_protocol);
}

# Ok::<(), packet_parser::ParsedPacketError>(())
```

## Testing Against Collisions

Protocol detection is not only about accepting valid packets. It is also about not accepting the wrong packets.

This is especially true when detection does not rely on ports. A payload may accidentally look like another protocol unless the parser checks enough structural constraints.

For that reason, the crate includes unit tests at two levels:

- Protocol-level tests that feed raw payloads to a specific parser.
- Packet-level tests that feed a full Ethernet frame to `PacketFlow`.

For example, the EtherNet/IP implementation includes a packet-level test with a full Ethernet + IPv4 + TCP frame using non-standard ports. The test verifies that the application layer is still detected as `EtherNet/IP`.

This kind of test is valuable because it exercises the parser the way users actually call it.

## Where This Crate Fits

`packet_parser` is useful if you are building:

- packet inspection tools
- PCAP analysis pipelines
- network protocol experiments
- educational tooling around packet formats
- industrial network monitoring prototypes
- Rust-based cybersecurity utilities

It is not trying to replace full-featured tools like Wireshark. Instead, it provides a Rust-native library that can be embedded into other programs.

That makes it useful when you want packet parsing as part of a larger system, not only as a standalone GUI workflow.

## Why Rust Works Well Here

Rust is a strong fit for packet parsing because it gives you:

- predictable memory behavior
- explicit error handling
- no garbage collector
- strong typing for protocol structures
- safe borrowing from packet buffers

Network packets are untrusted input. A parser should be careful by default. Rust does not make parsing logic correct automatically, but it does provide useful constraints. When combined with typed errors and focused tests, it helps keep the implementation maintainable.

## Roadmap Ideas

There is still plenty of room to grow.

Possible future directions include:

- deeper CIP parsing inside EtherNet/IP
- BACnet/IP support
- DNP3 support
- IEC 60870-5-104 support
- richer owned representations for long-term storage
- more PCAP-based regression tests
- more collision tests between application parsers

The crate is intentionally modular, so each new protocol can be added without changing the whole architecture.

## Final Thoughts

Packet parsing is a good example of where Rust's strengths are practical rather than theoretical. You need speed, structure, and careful handling of untrusted bytes. You also need code that can grow as protocol coverage expands.

`packet_parser` is a step in that direction: a layered, extensible Rust crate for decoding network frames, with increasing support for industrial protocols and real-world packet analysis workflows.

If you are interested in Rust, network analysis, or ICS protocol parsing, the project is open source and available here:

https://github.com/Akmot9/Packet-parser
