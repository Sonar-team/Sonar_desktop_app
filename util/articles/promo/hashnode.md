# Sonar: Map Your Network Without Touching It

> **Hashnode publishing notes**
> - In the post settings, set the **Canonical URL** to your first-published
>   version (Medium or your own blog) to avoid duplicate-content penalties.
> - Suggested tags: `cybersecurity`, `opensource`, `rust`, `networking`, `devops`.
> - Suggested subtitle: *"An open-source, passive, offline-first tool that turns
>   raw traffic into a trustworthy flow matrix — built for the networks other
>   tools aren't allowed to scan."*

There is a category of network that most security tools quietly avoid.

Industrial control systems where a single active scan can trip a PLC. Classified enclaves with no route to the internet. OT segments where "just run nmap" is a career-ending sentence. In these places the two questions that matter are **"Who is actually talking to whom?"** and **"Can I trust the tool — and its output — enough to put my name on the report?"**

That gap is exactly where **Sonar** lives.

## What Sonar is

Sonar is a lightweight, open-source desktop app built in **Rust** and **Tauri**. It listens to network traffic, reconstructs endpoint relationships in real time, and turns them into a **flow matrix** and an interactive **graph** — a defensible, comparable map of what your network is really doing. It runs on **Windows, Linux, and macOS** and *restitutes* what it observes: raw, traceable, exact. The human interprets and decides.

```
Passive capture  →  Flow matrix (SFMS)  →  Graph + labels  →  Import / export / merge
```

## Why it's different

- 🛰️ **Passive & offline by design** — emits **nothing** on the observed network, no online dependency. A listener, never a talker.
- 🏭 **Speaks industrial** — parses **Modbus, PROFINET, OPC UA, S7Comm, SNMP, EtherNet/IP** on top of the usual IT stack; decapsulates tunneled traffic (CAPWAP…) into linked parent/child flows.
- 🔐 **Verifiable, not just functional** — **reproducible builds**, **SBOM**, **Sigstore/cosign** signatures, provenance attestations. Proof, not faith.
- 🧭 **Draws no conclusions** — no invented data, no silent verdicts. Ambiguity goes back to the human. Fidelity first.

## Use cases

Network discovery & mapping · Shadow IT/OT discovery · Compliance & audit reporting · SOC baseline & rule tuning (feeds IDS/SIEM via the **SFMS** flow-matrix standard, mappable to IPFIX/NetFlow/NDR).

## Already in the wild

Across **100 public releases**, Sonar's binaries have been downloaded **2,300+ times**.

## Getting started (AGPLv3)

- **Windows** — install [Npcap](https://npcap.com/#download) (WinPcap-compatible mode), run `sonar.exe`.
- **Linux** — `sudo apt install libpcap-dev`, then `setcap cap_net_raw,cap_net_admin=eip`.
- **macOS** — libpcap included; nothing to install.

👉 **Releases & source:** https://github.com/Sonar-team/Sonar_desktop_app

If you own a network that can't be scanned but has to be understood, give Sonar an interface and let it listen. It won't touch your network. It'll just tell you the truth about it.
