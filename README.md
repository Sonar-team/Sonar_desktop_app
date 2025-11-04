<p align="center">
  <img src="src-tauri/icons/Square310x310Logo.png" alt="Sonar Logo" width="120" />
</p>

<h1 align="center">Sonar Desktop App</h1>

<p align="center">
  Lightweight and open-source desktop app built with Rust and Tauri.<br/>
  Captures network traffic and generates flow matrices for critical infrastructure auditing.
</p>

<p align="center">
  <a href="https://sonarcloud.io/summary/new_code?id=Sonar-team_Sonar_desktop_app">
    <img src="https://sonarcloud.io/api/project_badges/measure?project=Sonar-team_Sonar_desktop_app&metric=alert_status" alt="Quality Gate" />
  </a>
    <a href="https://app.fossa.com/projects/git%2Bgithub.com%2FSonar-team%2FSonar_desktop_app?ref=badge_large&issueType=license">
    <img src="https://app.fossa.com/api/projects/git%2Bgithub.com%2FSonar-team%2FSonar_desktop_app.svg?type=large&issueType=license" alt="FOSSA Status" />
  </a>
  <a href="https://github.com/Sonar-team/Sonar_desktop_app/releases">
    <img src="https://github.com/Sonar-team/Sonar_desktop_app/blob/main/util/livraison.png" alt="Releases" />
  </a>
  <a href="https://codecov.io/github/Sonar-team/Sonar_desktop_app">
    <img src="https://codecov.io/github/Sonar-team/Sonar_desktop_app/graph/badge.svg?token=UC4N2TUFRN" alt="Coverage" />
  </a>

</p>

---

## 🚀 Key Features

### 🧲 Packet Capture Engine

- Configures the selected network interface in **promiscuous mode**
- Reconstructs packet metadata in real time and maps traffic relationships
- Supports parsing of the following protocols:

  - Ethernet (MAC)
  - IPv4, IPv6, ARP
  - ICMPv4, ICMPv6
  - UDP, TCP
  - HTTP, DNS, TLS, SSL

---

## 🧰 System Dependencies

### Windows

- **NPcap:** Required for packet capture. You must also install the **WinPcap
  Developer Pack**.
- **Environment Variable:** Add the `/Lib` or `/Lib/x64` folder to your system
  `LIB` environment variable.

### Linux

- **libpcap-dev:** On Debian-based distributions, run:

  ```bash
  sudo apt install libpcap-dev
  ```
- **Non-root Execution:** Grant required capabilities using:

  ```bash
  sudo setcap cap_net_raw,cap_net_admin=eip path/to/binary
  ```

  Example:

  ```bash
  sudo setcap cap_net_raw,cap_net_admin=eip src-tauri/target/debug/sonar-desktop-app
  ```

### macOS

- **libpcap:** Already included by default on macOS systems. No additional setup
  is required.
