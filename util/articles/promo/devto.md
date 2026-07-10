---
title: "Sonar: Map Your Network Without Touching It"
published: false
description: "An open-source, passive, offline-first tool that turns raw traffic into a trustworthy flow matrix — built for the OT/ICS and air-gapped networks other tools aren't allowed to scan."
tags: cybersecurity, opensource, rust, networking
cover_image: ""
canonical_url: "https://medium.com/@YOUR_HANDLE/sonar-map-your-network-without-touching-it"
---

> ⚠️ Before publishing: set `canonical_url` to wherever you published FIRST
> (Medium or your own blog), flip `published` to `true`, and add a `cover_image`.
> The canonical URL is what keeps Google from treating this copy as duplicate
> content and penalizing it.

There is a category of network that most security tools quietly avoid.

Industrial control systems where a single active scan can trip a PLC. Classified enclaves with no route to the internet. OT segments where "just run nmap" is a career-ending sentence. In these places the two questions that matter aren't the ones your average scanner answers. They are:

**"Who is actually talking to whom?"** and, just as important, **"Can I trust the tool — and its output — enough to put my name on the report?"**

That gap is exactly where **Sonar** lives.

## What Sonar is

Sonar is a lightweight, open-source desktop application built in **Rust** and **Tauri**. It listens to network traffic, reconstructs the relationships between endpoints in real time, and turns them into a **flow matrix** and an interactive **graph** — a defensible, comparable map of what your network is really doing.

It runs on **Windows, Linux, and macOS**, ships as a small native binary, and does one thing with discipline: it *restitutes* what it observes. It draws no conclusions for you. The data is raw, traceable, and exact — the human interprets and decides.

```
Passive capture  →  Flow matrix (SFMS)  →  Graph + labels  →  Import / export / merge
```

## Why it's different

Plenty of tools capture packets. Four deliberate choices set Sonar apart:

### 🛰️ Passive and offline by design
In OT, active scanning is often forbidden. In classified networks, external connectivity is too. Sonar emits **nothing** on the network it observes and needs **no online dependency** to run. It is a listener, never a talker.

### 🏭 It speaks industrial
Beyond Ethernet, IPv4/IPv6, ARP, TCP/UDP, DNS, TLS and HTTP, Sonar parses the protocols that actually run a plant floor — **Modbus, PROFINET, OPC UA, S7Comm, SNMP, EtherNet/IP**. Tunneled traffic (CAPWAP and friends) is decapsulated and rendered as linked parent/child flows so nothing hides inside an overlay.

### 🔐 Verifiable, not just functional
Every release is built with **reproducible builds**, ships an **SBOM**, and is signed with **Sigstore/cosign** alongside **provenance attestations**. When you have to prove the binary running on an air-gapped network wasn't tampered with, Sonar hands you the evidence instead of asking for faith.

### 🧭 Sonar draws no conclusions
No invented data, no silent automatic verdicts. When something is ambiguous, Sonar hands the decision back to you rather than guessing. Fidelity first.

## What you can actually do with it

- **Network discovery & mapping** — surface devices and connections, build a full topology, expose the blind spots.
- **Shadow IT/OT discovery** — reveal unauthorized protocols and hidden IT, IoT, and OT devices or rogue connections.
- **Compliance & audit** — generate detailed, comparable network documentation and traffic reports, and track how the picture changes over time.
- **SOC rule creation & tuning** — establish a baseline of normal behavior, refine detection rules, feed the results into your IDS/SIEM.

## A format, not just a tool

Sonar is standardizing its output as the **Sonar Flow Matrix Standard (SFMS)** — interoperable and mappable onto the existing ecosystem (IPFIX / NetFlow / NDR). A matrix you can reuse anywhere beats a proprietary silo you can't.

## Already in the wild

Across **100 public releases**, Sonar's binaries have been downloaded **2,300+ times** — real auditors and OT teams pulling real builds onto real networks, release after release.

## Getting started

Free and open-source under **AGPLv3**:

- **Windows** — install [Npcap](https://npcap.com/#download) (enable WinPcap API-compatible mode), then run `sonar.exe`.
- **Linux** — `sudo apt install libpcap-dev`, then `setcap cap_net_raw,cap_net_admin=eip` to capture without root.
- **macOS** — libpcap ships with the OS; nothing extra to install.

👉 **Releases & source:** https://github.com/Sonar-team/Sonar_desktop_app

---

If you own a network that can't be scanned but has to be understood — an industrial line, an isolated enclave, an audit you'll have to defend — give Sonar an interface and let it listen. It won't touch your network. It'll just tell you the truth about it.

*Star it, break it, file an issue, send a PR.*
