# Tests E2E du binaire SONAR sous X11

`run-sonar-x11-e2e.sh` lance le vrai binaire Tauri release et reproduit les
parcours UI principaux avec XTest. Il utilise les dialogues de fichiers natifs,
les commandes IPC Rust et le WebView de production : ce n'est pas un test du
frontend avec un backend simulé.

Pour l'intégration GitLab CI sur une VM pilotée avec Ansible, consulter le
[runbook DevSecOps](./INTEGRATION_DEVSECOPS.md).

Le scénario couvre actuellement :

- le smoke test de démarrage et l'ouverture de la fenêtre ;
- les panneaux Configuration et Filtre BPF ;
- l'import PCAP puis le rendu du graphe ;
- l'activation et l'arrêt de ForceAtlas2 ;
- l'import et la gestion des labels ;
- les exports PNG, matrice CSV, labels CSV et logs (archive ZIP) ;
- le reset et l'import d'une matrice CSV ;
- l'absence de violation CSP et de panic dans les logs runtime ;
- la présence des images empaquetées lorsque `dist/` est disponible.

Les captures d'écran, exports, logs et le résumé sont conservés dans le dossier
d'artefacts. Toute vérification en échec termine le script avec un code non nul.

## Prérequis Linux

Exemple pour une VM Ubuntu/Debian, en complément des dépendances nécessaires au
binaire Tauri lui-même :

```bash
sudo apt-get install --yes \
  build-essential pkg-config libx11-dev libxtst-dev \
  xvfb openbox x11-utils wmctrl xclip imagemagick dbus-x11 file unzip
```

Le script utilise la session X11 courante lorsque `DISPLAY` est défini. Sinon,
il démarre son propre écran Xvfb en 1920×1080. `openbox` et `wmctrl` sont
utilisés lorsqu'ils sont disponibles ; le pilote sait aussi gérer directement
le focus X11 sur une image de VM minimale. Les répertoires de données,
configuration et cache de SONAR sont isolés dans les artefacts avec `XDG_*`.

## Exécution

Compiler puis tester :

```bash
./script/e2e/run-sonar-x11-e2e.sh \
  --build \
  --artifacts /tmp/sonar-e2e-artifacts
```

Tester un binaire déjà produit par la plateforme :

```bash
./script/e2e/run-sonar-x11-e2e.sh \
  --binary /chemin/vers/sonar \
  --artifacts "$CI_PROJECT_DIR/e2e-artifacts"
```

Les fixtures peuvent être remplacées sans modifier le script :

```bash
SONAR_E2E_PCAP=/fixtures/capture.pcap \
SONAR_E2E_MATRIX=/fixtures/matrice.csv \
SONAR_E2E_LABELS=/fixtures/labels.csv \
./script/e2e/run-sonar-x11-e2e.sh --binary ./sonar
```

## Capture réseau en direct

La capture live est désactivée par défaut : l'import PCAP permet de tester le
pipeline sans donner de privilèges réseau au job. Pour activer Démarrer/Arrêter,
le binaire de la VM doit posséder les deux capabilities :

```bash
sudo setcap cap_net_raw,cap_net_admin=eip ./sonar

./script/e2e/run-sonar-x11-e2e.sh \
  --binary ./sonar \
  --live-capture \
  --live-seconds 10
```

Le script refuse explicitement ce mode si les capabilities ne sont pas
présentes. Une reconstruction ou un remplacement du binaire retire généralement
les capabilities : il faut donc exécuter `setcap` après le build.

## Artefacts produits

- `summary.txt` : statut synthétique et fixtures utilisées ;
- `runtime.log` et `startup-smoke.log` : sorties du binaire ;
- `01-startup.png` à `10-matrix-imported.png` : preuves visuelles ;
- `graph-export.png`, `matrix-export.csv`, `labels-export.csv` ;
- `logs-export.log/` : dossier créé par l'export actuel des logs ;
- `xdg-data/`, `xdg-config/`, `xdg-cache/` : état isolé de la session.

Ce harness cible Linux/X11. Les validations Windows et macOS devront utiliser
les pilotes natifs prévus séparément dans l'epic `e2e-installateurs`.
