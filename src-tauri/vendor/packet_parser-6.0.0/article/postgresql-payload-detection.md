# Detecting PostgreSQL Traffic by Payload in Rust

PostgreSQL is often associated with TCP port `5432`, but a packet parser should not rely on that assumption too strongly.
Ports are useful metadata, not protocol proof.

In real captures, PostgreSQL can appear on non-standard ports, behind proxies, inside test harnesses, or in replayed traffic where port numbers are not reliable.
At the same time, blindly trying to parse every TCP payload as PostgreSQL creates another problem: false positives.

This article explains how `packet_parser` moved PostgreSQL recognition from port-based detection to payload-based detection while keeping the parser conservative.

Repository: https://github.com/Akmot9/Packet-parser

## The Problem With Port-Based Detection

The first PostgreSQL integration in `packet_parser` recognized PostgreSQL at the application layer when either TCP endpoint used the standard PostgreSQL port.

That was simple and useful:

- parse Ethernet
- parse IPv4 or IPv6
- parse TCP
- if source or destination port is `5432`, try the PostgreSQL parser

The limitation is obvious once captures stop being ideal.

A valid PostgreSQL payload on port `15432` would be ignored.
A local test service could use an ephemeral port.
A proxy could terminate or forward traffic using a different port.

For a packet parser, that is too rigid.
The application layer should be identified from the bytes when the protocol provides enough structure to do that safely.

## Why Payload Detection Is Harder

Removing the port check is easy.
Doing it without false positives is the real work.

PostgreSQL messages are length-prefixed, and many typed messages start with a single ASCII byte:

- `Q` for Query
- `P` for Parse
- `B` for Bind
- `S` for ParameterStatus or Sync
- `Z` for ReadyForQuery
- `K` for BackendKeyData

That shape is not unique enough by itself.
Random application data can accidentally start with one of those bytes followed by a plausible length field.

Some messages are especially weak as standalone evidence.
For example, a single `Sync` message is only five bytes:

```text
53 00 00 00 04
```

That is valid PostgreSQL, but it is not strong enough to identify an arbitrary TCP payload as PostgreSQL.

The new detection path therefore separates two questions:

- Can the payload be parsed as PostgreSQL?
- Does the payload contain enough PostgreSQL-specific evidence to label it as PostgreSQL automatically?

Only the second question is used for automatic application detection.

## Strong Evidence

The detector now parses the payload first, then checks for stronger protocol evidence.

Examples of strong evidence include:

- a StartupMessage with known startup parameters such as `user`, `database`, or `client_encoding`
- a Query or Parse message whose query starts with a known SQL keyword
- backend ParameterStatus messages with known names such as `server_version`, `DateStyle`, or `session_authorization`
- structured ErrorResponse or NoticeResponse fields
- SASL Authentication messages with valid C-string mechanism lists
- SSLRequest, GSSENCRequest, or CancelRequest shapes

This keeps valid PostgreSQL traffic detectable without relying on `5432`, while avoiding weak matches that only happen to fit the framing.

## Compatible But Not Sufficient

Some messages are valid and useful once PostgreSQL is already known, but they are not enough by themselves to classify unknown traffic.

Examples:

- `ReadyForQuery`
- `BackendKeyData`
- empty `Sync`
- raw typed messages without protocol-specific fields

The parser still accepts these messages through `PostgreSqlPacket::try_from`.
That matters because callers may already know they are parsing PostgreSQL, or they may be decoding a stream after earlier packets established the protocol.

But automatic `PacketFlow` detection requires at least one stronger signal.

That distinction reduces false positives without weakening the protocol parser itself.

## Startup Messages

PostgreSQL startup packets have a different shape from regular typed messages.
They do not start with a message type byte.
Instead, the payload begins with a length field followed by a protocol version or special startup code.

The parser now accepts both:

- protocol `3.0` (`196608`)
- protocol `3.2` (`196610`)

That matters because the frontend/backend protocol can evolve while keeping the same basic startup framing.

A typical startup message includes key/value parameters:

```text
user\0oryx\0database\0mailstore\0\0
```

Those fields are strong evidence because they are PostgreSQL-specific and require valid NUL-terminated UTF-8 strings.

## Backend Responses

Backend traffic also provides good detection signals.

A real captured backend packet can contain several PostgreSQL messages in a single TCP segment:

- AuthenticationOk
- ParameterStatus: `client_encoding = UNICODE`
- ParameterStatus: `DateStyle = ISO, MDY`
- ParameterStatus: `is_superuser = off`
- ParameterStatus: `server_version = 7.4.6`
- ParameterStatus: `session_authorization = oryx`
- BackendKeyData
- ReadyForQuery

That is much stronger than a single byte tag.
The parser validates each message length, walks the sequence, and checks that the structured fields make sense.

The important part is that `BackendKeyData` and `ReadyForQuery` are accepted as part of the sequence, but the detection confidence comes from the surrounding ParameterStatus messages.

## CancelRequest And Secret Keys

The PostgreSQL CancelRequest format also deserves care.

Older assumptions often treat the secret key as a fixed `u32`.
Current PostgreSQL protocol documentation describes it as bytes whose length is part of the message body.

The parser now keeps the secret key as a borrowed byte slice:

```rust
CancelRequest {
    process_id: u32,
    secret_key: &'a [u8],
}
```

That keeps the implementation closer to the protocol format and avoids copying variable data.

## PacketFlow Integration

At the full-packet level, detection now looks like this conceptually:

```rust
if transport.protocol == TransportProtocol::Tcp
    && is_likely_postgresql_payload(payload)
{
    return Some(Application {
        application_protocol: "PostgreSQL".to_string(),
    });
}
```

There is no `5432` requirement.
The TCP port can still be useful metadata, but it is no longer the proof.

## Testing With Real Frames

Two packet-level tests were added from real captures.

The first is a PostgreSQL startup packet:

- Ethernet
- IPv4 loopback
- TCP `45930 -> 5432`
- 38 bytes of PostgreSQL payload
- StartupMessage with `user = oryx` and `database = mailstore`

The second is a backend response packet:

- Ethernet
- IPv4 loopback
- TCP `5432 -> 45931`
- 161 bytes of PostgreSQL payload
- 8 PostgreSQL messages in one segment
- AuthenticationOk, ParameterStatus messages, BackendKeyData, and ReadyForQuery

These tests matter because they exercise the full parsing path:

- Ethernet header parsing
- IPv4 parsing
- TCP header and TCP options parsing
- application payload extraction
- PostgreSQL detection
- PostgreSQL message decoding

That is more valuable than only testing raw application bytes.
Users usually call the parser with complete frames, so the tests should include complete frames.

## Why This Is Safer

The new detection strategy is stricter than port-based detection in one sense and more flexible in another.

It is more flexible because PostgreSQL can be detected on non-standard TCP ports.
It is stricter because a port number alone no longer makes a weak payload PostgreSQL.

That tradeoff fits the goal of `packet_parser`:

- preserve useful layers when possible
- avoid overclaiming application protocols
- parse known protocols with typed structures
- keep detection explainable and testable

## Closing Note

Application protocol detection is not just about recognizing more traffic.
It is also about refusing to recognize the wrong traffic.

PostgreSQL has enough structure to support payload-based detection, but only if the detector checks more than the first byte and a length field.

By separating "this parses as PostgreSQL" from "this is strong enough to identify PostgreSQL automatically", `packet_parser` gets better behavior on both sides: non-standard-port PostgreSQL is detected, and weak accidental matches are ignored.
