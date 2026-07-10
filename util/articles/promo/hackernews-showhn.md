# Hacker News — Show HN

> HN etiquette: no marketing language, no emoji, no hype adjectives. State what
> it is, be technically honest, mention it's open-source and passive. Post the
> GitHub repo as the URL. Then add the text below as the first comment.
> Best time: weekday ~08:00–10:00 US Eastern. Be around to answer replies.

## Title (80 char max — pick one)

- `Show HN: Sonar – Passive network mapping for OT/ICS and air-gapped networks`
- `Show HN: Sonar – Turn passive traffic capture into an auditable flow matrix`

## URL
https://github.com/Sonar-team/Sonar_desktop_app

## First comment (author context)

Sonar is an open-source (AGPLv3) desktop app — Rust + Tauri — for mapping networks you're not allowed to actively scan: industrial/OT segments, air-gapped or classified enclaves, audit engagements where nmap is off the table.

It's strictly passive: it captures on a chosen interface in promiscuous mode, reconstructs endpoint relationships, and emits a flow matrix plus an interactive graph. It never transmits on the observed network, and it runs fully offline (Rust deps are vendored; builds are reproducible offline), which is the whole point for air-gapped use.

Beyond the usual IT stack (Ethernet, IPv4/6, ARP, TCP/UDP, DNS, TLS, HTTP) it parses industrial protocols — Modbus, PROFINET, OPC UA, S7Comm, SNMP, EtherNet/IP — and decapsulates tunneled traffic (e.g. CAPWAP) into linked parent/child flows.

Two design choices I'd genuinely like feedback on:

1. Supply-chain assurance is a first-class feature, not an afterthought: reproducible builds, SBOM, Sigstore/cosign signatures, and provenance attestations ship with every release, because in these environments you have to prove the binary wasn't tampered with.

2. It deliberately draws no conclusions. No automatic threat verdicts. It restitutes observed flows as raw, traceable data; the human interprets. On ambiguity (e.g. label conflicts on merge) it hands the decision back rather than guessing.

The output format (Sonar Flow Matrix Standard / SFMS) is meant to be interoperable and mappable to IPFIX/NetFlow so it can feed an IDS/SIEM instead of being a silo.

Caveats: it ships as a raw binary, not a polished installer; packet capture needs Npcap on Windows / libpcap on Linux/macOS and the usual capture privileges. Happy to answer questions about the capture engine, the flow-matrix model, or the offline/reproducible build setup.
