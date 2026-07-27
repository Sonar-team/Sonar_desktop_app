---
name: verify
description: Vérifier un changement SONAR avec le niveau adapté (contrôles statiques, binaire Tauri release, smoke test, E2E X11, capture live ou reproductibilité). Utiliser après une modification Vue/TypeScript, Rust/Tauri, IPC, CSP/assets, capture réseau ou chaîne de release, et fournir des preuves par codes de sortie, logs et artefacts.
---

# Vérifier SONAR

Choisir le niveau le plus élevé justifié par le changement. Ne jamais conclure
qu'une fonctionnalité marche uniquement parce que le processus reste ouvert.

## 1. Contrôles de base

Pour tout changement frontend :

```bash
deno task typecheck
deno task lint
deno task test
deno task build
```

Pour un changement Rust, ajouter :

```bash
(cd src-tauri && cargo test --locked)
```

## 2. Binaire release et smoke test

Pour un changement Tauri, IPC, CSP, assets ou démarrage, construire comme la
release puis utiliser le smoke test maintenu :

```bash
deno run -A ./security/repro-env.ts run \
  deno task tauri build --ci --no-sign --no-bundle

./script/ci/smoke-test-release-binary.sh \
  ./src-tauri/target/release/sonar 30
```

Exiger un code de sortie nul et le marqueur `SONAR_STARTUP_VALIDATION=OK`.
Le smoke test valide le démarrage natif ; il ne valide pas les parcours UI.

## 3. Parcours utilisateur E2E sous Linux/X11

Pour une fonctionnalité visible, un import/export, le graphe, les labels ou
une interaction WebView/IPC, exécuter le vrai binaire avec le harnais X11 :

```bash
./script/e2e/run-sonar-x11-e2e.sh \
  --binary ./src-tauri/target/release/sonar \
  --artifacts /tmp/sonar-e2e-artifacts
```

Le harnais gère le PID, le nettoyage, un écran Xvfb si nécessaire et des
répertoires `XDG_*` isolés. Il vérifie notamment le démarrage, la configuration,
le filtre BPF, les imports PCAP/matrice/labels, le graphe, les exports, les
assets empaquetés ainsi que l'absence de violation CSP et de panic.

Conserver comme preuves :

- `summary.txt` avec `SONAR X11 E2E: PASS` ;
- `runtime.log` et `startup-smoke.log` ;
- les captures d'écran et fichiers exportés ;
- le code de sortie nul du script.

## 4. Capture réseau en direct

Ne pas donner de capabilities pour un test standard : l'import PCAP couvre le
pipeline sans privilège réseau. Pour vérifier explicitement Démarrer/Arrêter,
appliquer les capabilities après le build puis activer le scénario dédié :

```bash
sudo setcap cap_net_raw,cap_net_admin=eip \
  ./src-tauri/target/release/sonar

./script/e2e/run-sonar-x11-e2e.sh \
  --binary ./src-tauri/target/release/sonar \
  --live-capture \
  --artifacts /tmp/sonar-e2e-live-artifacts
```

Une reconstruction ou un remplacement du binaire retire généralement les
capabilities. Ne jamais les appliquer avant le dernier build.

## 5. Reproductibilité et release

Pour un changement de dépendance, d'outillage, de configuration Tauri ou de
chaîne de publication, ajouter le contrôle reproductible isolé :

```bash
ISOLATED=1 ./security/repro-check.sh
```

Sous Windows, utiliser `pwsh -File security/repro-check.ps1`. La comparaison
porte sur le binaire non signé, pas sur tous les installateurs natifs.

## Interpréter correctement les logs

- Les appels `info`, `warn` et `error` de `@tauri-apps/plugin-log` fournissent
  une preuve persistante.
- `attachConsole()` relaie les logs du plugin vers la console WebView ; il ne
  capture pas automatiquement tous les `console.error`, exceptions JavaScript
  ou rejets de promesse.
- `[CPU.vue] Listener registered` prouve seulement que le composant est monté
  et que son abonnement Tauri a réussi. Ce marqueur ne prouve pas à lui seul
  le fonctionnement de tous les IPC, assets ou parcours utilisateur.
- Une URL `tauri://localhost` montre que le WebView utilise le protocole Tauri,
  mais ne prouve pas seule l'absence de violation CSP. Utiliser le contrôle E2E.

## Sécurité d'exécution et compte rendu

- Ne pas utiliser `kill %1`, `pkill` ou un `pgrep` large pour arrêter SONAR.
  Laisser les scripts maintenus gérer leurs propres processus.
- Ne pas interrompre une instance utilisateur. Utiliser une session ou un
  `DISPLAY` dédié lorsque SONAR est déjà ouvert.
- Tester CSP et assets sur un binaire release, jamais uniquement avec Vite.
- Signaler les commandes exécutées, les niveaux omis avec leur raison, les
  codes de sortie et les chemins des artefacts. Ne déclarer la vérification
  réussie que si toutes les preuves attendues sont présentes.

Le harnais complet cible Linux/X11. Ne pas extrapoler son résultat aux
installateurs Windows ou macOS sans exécuter leurs validations natives.
