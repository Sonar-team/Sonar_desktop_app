[![Quality Gate Status](https://sonarcloud.io/api/project_badges/measure?project=Sonar-team_Sonar_desktop_app&metric=alert_status)](https://sonarcloud.io/summary/new_code?id=Sonar-team_Sonar_desktop_app)

# Tauri + Vue 3

This template should help get you started developing with Tauri + Vue 3 in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
# Sonar_desktop_app

## Fonctionnalités
+ **Interception** : configure l'adaptateur réseau en mode promiscue et reconstruit toutes les informations liées aux paquets collectés. Pour l'instant, la liste des paquets entièrement analysés comprend :
    - Ethernet
    - IPv4, IPv6, arp
    - ICMPv4, ICMPv6
    - UDP, TCP
    - HTTP, DNS, TLS
