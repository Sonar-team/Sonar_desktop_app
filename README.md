[![Quality Gate Status](https://sonarcloud.io/api/project_badges/measure?project=Sonar-team_Sonar_desktop_app&metric=alert_status)](https://sonarcloud.io/summary/new_code?id=Sonar-team_Sonar_desktop_app)

<a href="https://github.com/Sonar-team/Sonar_desktop_app/releases">
  <img src="https://raw.githubusercontent.com/Sonar-team/Sonar_desktop_app/main/assets/images/livraison.png" alt="Release" width="100">
</a>

[![codecov](https://codecov.io/github/Sonar-team/Sonar_desktop_app/graph/badge.svg?token=UC4N2TUFRN)](https://codecov.io/github/Sonar-team/Sonar_desktop_app)

[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FSonar-team%2FSonar_desktop_app.svg?type=shield&issueType=license)](https://app.fossa.com/projects/git%2Bgithub.com%2FSonar-team%2FSonar_desktop_app?ref=badge_shield&issueType=license)

# Application de bureau Sonar
![logo](src-tauri/icons/Square310x310Logo.png)
## Fonctionnalités Clés

### Interception
- **Fonctionnalité d'Interception :** Cette fonction configure l'adaptateur réseau en mode promiscue et reconstruit de manière exhaustive les informations des paquets capturés. Les paquets actuellement pris en charge pour une analyse complète comprennent :
  - Ethernet
  - IPv4, IPv6, ARP
  - ICMPv4, ICMPv6
  - UDP, TCP
  - HTTP, DNS, TLS

## Dépendances Système

### Pour Windows
- **NPcap :** Nécessaire pour le fonctionnement de l'application. Il faut également installer le Pack de Développeur WinPcap.
- **Configuration de l'Environnement :** Ajoutez le dossier `/Lib` ou `/Lib/x64` à votre variable d'environnement `LIB`.

### Pour Linux
- **libpcap-dev :** Sur les distributions basées sur Debian, installez `libpcap-dev`.
- **Permissions d'Exécution :** Si l'application n'est pas exécutée en tant que root, configurez les capacités système avec la commande `sudo setcap cap_net_raw,cap_net_admin=eip chemin/vers/bin`.
Commande pour les permissions spéciales sous Linux : `sudo setcap cap_net_raw,cap_net_admin=eip src-tauri/target/debug/sonar-desktop-app`

### Pour Mac OS X
- **libpcap :** Cette bibliothèque est généralement préinstallée sur Mac OS X.
