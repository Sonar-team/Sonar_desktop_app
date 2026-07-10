# Sonar: Map Your Network Without Touching It

### An open-source, passive, offline-first tool that turns raw traffic into a trustworthy flow matrix — built for the networks other tools aren't allowed to scan.

![Sonar](../../src-tauri/icons/Square310x310Logo.png)

---

There is a category of network that most security tools quietly avoid.

Industrial control systems where a single active scan can trip a PLC. Classified enclaves with no route to the internet. OT segments where "just run nmap" is a career-ending sentence. In these places the two questions that matter aren't the ones your average scanner answers. They are:

**"Who is actually talking to whom?"** and, just as important, **"Can I trust the tool — and its output — enough to put my name on the report?"**

That gap is exactly where **Sonar** lives.

---

## What Sonar is

Sonar is a lightweight, open-source desktop application built in **Rust** and **Tauri**. It listens to network traffic, reconstructs the relationships between endpoints in real time, and turns them into a **flow matrix** and an interactive **graph** — a defensible, comparable map of what your network is really doing.

It runs on **Windows, Linux, and macOS**, ships as a small native binary, and does one thing with discipline: it *restitutes* what it observes. It draws no conclusions for you. The data is raw, traceable, and exact — the human interprets and decides.

```
Passive capture  →  Flow matrix (SFMS)  →  Graph + labels  →  Import / export / merge
```

## Why it's different

Plenty of tools capture packets. Four deliberate choices set Sonar apart:

### 🛰️ Passive and offline by design
In OT, active scanning is often forbidden. In classified networks, external connectivity is too. Sonar emits **nothing** on the network it observes and needs **no online dependency** to run. It is a listener, never a talker — a constraint we treat as the headline feature, not the fine print.

### 🏭 It speaks industrial
Beyond Ethernet, IPv4/IPv6, ARP, TCP/UDP, DNS, TLS and HTTP, Sonar parses the protocols that actually run a plant floor — **Modbus, PROFINET, OPC UA, S7Comm, SNMP, EtherNet/IP**. This is a tool designed for OT/ICS, not just office IT. Tunneled traffic (CAPWAP and friends) is decapsulated and rendered as linked parent/child flows so nothing hides inside an overlay.

### 🔐 Verifiable, not just functional
Trust isn't declared, it's proven. Every release is built with **reproducible builds**, ships an **SBOM**, and is signed with **Sigstore/cosign** alongside **provenance attestations**. When you have to demonstrate that the binary running on an air-gapped network wasn't tampered with, Sonar hands you the evidence instead of asking for faith.

### 🧭 Sonar draws no conclusions
No invented data, no silent automatic verdicts. When something is ambiguous — a label conflict, an overlapping flow — Sonar hands the decision back to you rather than guessing. Fidelity first.

## What you can actually do with it

- **Network discovery & mapping** — automatically surface devices and connections, build a full topology, and expose the blind spots.
- **Shadow IT/OT discovery** — reveal unauthorized protocols and hidden IT, IoT, and OT devices or rogue connections.
- **Compliance & audit** — generate detailed, comparable network documentation and traffic reports for regulators, and track how the picture changes over time.
- **SOC rule creation & tuning** — establish a baseline of normal behavior and use it to write or refine detection rules, then feed the results into your IDS/SIEM.

If your title is Network Administrator, Network Auditor, SOC Architect, or OT/ICS lead, this was built for your day.

## A format, not just a tool

A flow matrix is only worth exchanging if its format is stable. Sonar is standardizing its output as the **Sonar Flow Matrix Standard (SFMS)** — designed to be interoperable and mappable onto the existing ecosystem (IPFIX / NetFlow / NDR). The goal is deliberately un-selfish: a matrix you can reuse anywhere beats a proprietary silo you can't.

## Quietly, people are already using it

Sonar isn't a slideware project. Across **100 public releases**, its binaries have been downloaded **2,300+ times** — real auditors and OT teams pulling real builds onto real networks, release after release. It's shipping, it's iterating, and the latest version keeps tightening the screws (the 4.3.x line eliminated every avoidable panic in production code and enforces it in CI).

## Getting started

Sonar is free and open-source under **AGPLv3**. Grab a release binary for your platform and point it at an interface:

- **Windows** — install [Npcap](https://npcap.com/#download) (enable WinPcap API-compatible mode), then run `sonar.exe`.
- **Linux** — `sudo apt install libpcap-dev`, then grant capture rights without root via `setcap cap_net_raw,cap_net_admin=eip`.
- **macOS** — libpcap ships with the OS; nothing extra to install.

👉 **Releases & source:** [github.com/Sonar-team/Sonar_desktop_app](https://github.com/Sonar-team/Sonar_desktop_app)

---

If you own a network that can't be scanned but has to be understood — an industrial line, an isolated enclave, an audit you'll have to defend — give Sonar an interface and let it listen. It won't touch your network. It'll just tell you the truth about it.

*Sonar is open-source. Star it, break it, file an issue, send a PR. The map gets better when more people are drawing it.*
