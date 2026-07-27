<p align="center">
  <img src="src-tauri/icons/Square310x310Logo.png" alt="Sonar Logo" width="120" />
</p>

<h1 align="center">Sonar Desktop App</h1>

<p align="center">
  Lightweight and open-source desktop app built with Rust and Tauri.<br/>
  Captures network traffic and generates flow matrices for critical infrastructure auditing.
</p>

<p align="center">
  <i>
    Gain deep network visibility.<br>
    Eliminate shadow IT/OT.<br>
    Empower your SOC/CSIRT teams.<br>
    Stay ahead of cyber threats.
  </i>
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

## For:

- Network Administrators
- Network Auditors
- SOC Architects

## Use Cases

### Network Discovery & Mapping

Automatically discover devices and connections. Build complete topology maps to
identify blind spots and ensure full network coverage.

### Shadow IT/OT Discovery

Identify unauthorized network protocols, as well as hidden IT, IoT, and OT
devices or rogue connections.

### Compliance & Audit Support

Generate detailed network documentation and traffic reports for regulatory
audits. Track changes over time and demonstrate visibility controls.

### SOC Rule Creation and Tuning

Establish a baseline of normal network behavior and use it to create or refine
SOC rules.

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

### 📄 Format des matrices de flux

Le format des matrices produites et consommées par SONAR (colonnes,
préambule versionné, tunnels, provenance, règles d'import) est décrit dans
[`SFMS.md`](./SFMS.md).

### 🔗 Types de liaison (LINKTYPE/DLT) supportés

Chaque paquet est parsé avec le décodeur de son type de liaison réel —
jamais « supposé Ethernet » :

| LINKTYPE | Valeur | Capture live | Import PCAP | Réimport matrice CSV/XLSX |
|---|---|---|---|---|
| `ETHERNET` | 1 | ✅ | ✅ | ✅ |
| `RAW` (IP nu) | 101 (DLT 12 en live) | ✅ | ✅ | ✅ |
| `LINUX_SLL` (cooked v1, `-i any`) | 113 | ✅ | ✅ | ✅ |
| `LINUX_SLL2` (cooked v2) | 276 | ✅ | ✅ | ✅ |

Limites, toutes **explicites** (jamais de dégradation silencieuse) :

- un DLT sans décodeur est refusé avant toute mutation du relevé (au
  démarrage de capture comme à l'import) ;
- **un relevé = un réseau = un DLT** : la fusion de sources de DLT
  différents est refusée (matrices, PCAP multi-fichiers, capture sur une
  interface d'un autre DLT que le relevé en cours) ;
- l'export/réimport RAW, SLL et SLL2 préserve exactement l'identité SFMS
  de chaque conversation ;
- pour SLL/SLL2, l'adresse source et le protocole font partie de cette
  identité. `packet_type`, `hardware_type`, `address_length`,
  `reserved_mbz` et `interface_index` décrivent le point d'observation :
  ils restent fidèles dans les paquets affichés, mais ne créent pas de lignes
  supplémentaires dans la matrice et ne sont pas ajoutés au CSV ;
- aucune colonne `link_details` n'est nécessaire : une même conversation
  importée depuis plusieurs sondes fusionne, et `origin` conserve les noms
  des fichiers qui l'ont observée ;
- un LINKTYPE sans projection d'identité SFMS définie est refusé au réimport
  plutôt que reconstruit sous un autre DLT ;
- un PCAPNG multi-interfaces à DLT ou snaplens mélangés est refusé par
  libpcap à la lecture ;
- les matrices exportées portent leur DLT dans la ligne préambule
  `#SFMS version=1 dlt=…` (un export antérieur au préambule est réimporté
  en Ethernet implicite).

---

## Release Binaries

Release assets contain raw binaries and native bundles: DMG on macOS, DEB/RPM
on Linux and NSIS setup on Windows.

## Configuration minimale

- **Systeme 64 bits:** Windows 10/11, Linux x86_64 recent ou macOS recent.
- **Processeur:** 2 coeurs minimum.
- **Memoire:** 4 Go de RAM minimum, 8 Go recommandes pour les captures a fort
  debit ou les analyses longues.
- **Stockage:** 500 Mo libres pour l'application, plus l'espace necessaire aux
  exports et fichiers de capture.
- **Capture reseau:** droits administrateur/root ou capabilities Linux
  `cap_net_raw` et `cap_net_admin` sur le binaire.
- **Pilote/librairie de capture:** Npcap sous Windows, libpcap sous Linux,
  libpcap systeme sous macOS.

### Windows

- Npcap is **not included in SONAR**. Install it separately from the
  [official Npcap download page](https://npcap.com/#download) before launching
  `sonar.exe`.
- During Npcap installation, enable **WinPcap API-compatible Mode**.
- The NSIS installer checks this prerequisite and offers to open the official
  download page when Npcap is absent or incompatible.
- Because the Windows binary links to Npcap's runtime DLLs, SONAR may not start
  until a compatible Npcap installation is available.

## 🧰 System Dependencies

### Windows

- **Npcap:** Required at runtime for packet capture.
- **Build from source:** The SDK import libraries required by the linker are
  versioned in the repository; they do not include or install the Npcap driver.

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
