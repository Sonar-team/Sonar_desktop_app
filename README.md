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
- **WinPcap :** Nécessaire pour le fonctionnement de l'application. Il faut également installer le Pack de Développeur WinPcap.
- **Configuration de l'Environnement :** Ajoutez le dossier `/Lib` ou `/Lib/x64` à votre variable d'environnement `LIB`.

### Pour Linux
- **libpcap-dev :** Sur les distributions basées sur Debian, installez `libpcap-dev`.
- **Permissions d'Exécution :** Si l'application n'est pas exécutée en tant que root, configurez les capacités système avec la commande `sudo setcap cap_net_raw,cap_net_admin=eip chemin/vers/bin`.
Commande pour les permissions spéciales sous Linux : `sudo setcap cap_net_raw,cap_net_admin=eip src-tauri/target/debug/sonar-desktop-app`

### Pour Mac OS X
- **libpcap :** Cette bibliothèque est généralement préinstallée sur Mac OS X.

# Tauri + Vue 3

Ce modèle est conçu pour faciliter le développement d'applications avec Tauri et Vue 3 en utilisant Vite. Il utilise les Composants de Fichier Unique (SFC) de Vue 3 avec `<script setup>`. Pour plus d'informations sur cette fonctionnalité, veuillez consulter la [documentation sur le script setup de Vue 3](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup).

## Configuration IDE Recommandée

Pour une expérience de développement optimale, il est recommandé d'utiliser :
- [VS Code](https://code.visualstudio.com/), un éditeur de code puissant et polyvalent.
- [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar), une extension pour le support de Vue.
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode), une extension pour faciliter le développement Tauri dans VS Code.
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer), une extension offrant un support avancé pour le langage Rust.
