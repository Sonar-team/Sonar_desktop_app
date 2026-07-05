---
name: verify
description: Vérifier un changement Sonar en conditions réelles (build release, lancement, évidence par logs)
---

# Vérifier Sonar en runtime

## Build + lancement

```bash
deno task tauri build --ci --no-sign --no-bundle   # embarque dist/ + CSP dans le binaire
./src-tauri/target/release/sonar > /tmp/sonar-test.log 2>&1 &
sleep 15 && kill %1
```

## Évidence

Le webview forwarde sa console dans les logs (`[webview][INFO/ERROR] ...`),
c'est la source d'évidence principale :

- `[CPU.vue] Listener registered` → JS chargé + événements IPC OK.
- Toute erreur JS/IPC apparaît en `[webview][ERROR]` avec l'URL
  `tauri://localhost/assets/...` (preuve que les assets passent par le
  protocole tauri, donc sous CSP).

## Pièges connus

- **Une seule instance à la fois** : le raccourci global Ctrl+C fait paniquer
  toute seconde instance (`HotKey already registered`). Vérifier
  `pgrep -f "target/(debug|release)/sonar"` avant de lancer — un
  `deno task tauri dev` de l'utilisateur compte.
- La CSP et les assets embarqués ne s'appliquent **pas** en mode dev
  (servis par Vite) : tester sur le binaire release.
- La capture réseau échoue sans capabilities :
  `sudo setcap cap_net_raw,cap_net_admin=eip src-tauri/target/release/sonar`.
- Screenshot GNOME Shell via D-Bus refusé (`Screenshot is not allowed`) ;
  pas de Xvfb/scrot sur la machine — l'évidence passe par les logs.
