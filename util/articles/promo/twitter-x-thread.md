# X / Twitter thread

> Keep each tweet under 280 chars. Attach a screenshot/GIF of the graph view to
> tweet 1 — it roughly doubles engagement. Pin the thread after posting.

**1/**
Some networks can't be scanned.

ICS where an active scan trips a PLC. Air-gapped enclaves. OT where "just run nmap" ends careers.

We built Sonar for exactly those networks 🛰️

Open-source, passive, offline. It listens — and never talks back.
🧵

**2/**
Sonar (Rust + Tauri) turns raw traffic into a trustworthy flow matrix + interactive graph.

The rule it never breaks: it emits NOTHING on the network it observes.

A listener, never a talker.

**3/**
It speaks industrial, not just office IT:

Modbus · PROFINET · OPC UA · S7Comm · SNMP · EtherNet/IP

Tunneled traffic (CAPWAP…) gets decapsulated into linked parent/child flows. Nothing hides in an overlay.

**4/**
Trust isn't declared, it's proven.

Every release:
🔐 reproducible builds
📦 SBOM
✍️ Sigstore/cosign signatures + provenance

So you can prove the binary on your air-gapped net wasn't tampered with.

**5/**
Philosophy: Sonar draws no conclusions.

No invented data. No silent verdicts. When something's ambiguous, the decision goes back to YOU.

Fidelity first. The tool restitutes; the human interprets.

**6/**
Use it for:
• Network discovery & mapping
• Shadow IT/OT hunting
• Compliance & audit evidence
• SOC baselines → IDS/SIEM rules (via the SFMS flow-matrix standard)

**7/**
Already in the wild: 100 releases, 2,300+ downloads.

Free & open-source (AGPLv3). Windows / Linux / macOS.

Give it an interface and let it listen. It won't touch your network — it'll just tell you the truth about it.

⭐ https://github.com/Sonar-team/Sonar_desktop_app
