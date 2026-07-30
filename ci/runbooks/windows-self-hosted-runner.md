# Runbook : runner Windows self-hosted (Npcap, #146 phase 1)

Provisionnement du runner GitHub Actions self-hosted utilisé par le job
`windows-e2e-npcap` (`.github/workflows/publish-smoke.yml`) pour installer
réellement le paquet NSIS de release et vérifier que SONAR détecte Npcap au
démarrage. Contexte et justification licence : `security/licences.md`
(décision #138, entrée du 30/07/2026).

Ce runner est **persistant** (pas éphémère) : Npcap y est installé une seule
fois, à la main ; le job CI ne fait qu'installer/désinstaller le paquet
SONAR à chaque exécution (le script `script/ci/run-sonar-windows-e2e.ps1` se
charge du nettoyage avant/après).

## 1. Provisionner la VM (Proxmox)

- VM Windows (Server ou 10/11), 2 vCPU / 4 Go RAM suffisent.
- Réseau : accès sortant à `github.com`/`*.actions.githubusercontent.com`
  (pas besoin d'entrant).
- PowerShell 5.1+ (préinstallé) suffit ; `pwsh` (PowerShell 7) recommandé si
  disponible, le workflow utilise `shell: pwsh`.

## 2. Installer Npcap manuellement

1. Télécharger depuis <https://npcap.com/#download> (édition gratuite).
2. Lancer l'installeur **à la main**, cocher **« WinPcap API-compatible
   Mode »** (requis par SONAR, cf. `src-tauri/windows/hooks.nsh`).
3. Ne jamais scripter/automatiser cette étape — c'est précisément ce qui
   maintient l'usage dans le cadre de la licence Npcap gratuite (pas
   d'installation silencieuse/automatisée). Voir `security/licences.md`
   pour le raisonnement complet.

## 3. Enregistrer le runner GitHub Actions

Dans le dépôt : **Settings → Actions → Runners → New self-hosted runner**,
choisir Windows, suivre les commandes fournies (`config.cmd` ou
`config.ps1`) avec l'URL et le token affichés. À l'étape des labels,
ajouter :

```
self-hosted, windows, npcap
```

Ce sont exactement les labels attendus par `runs-on: [self-hosted, windows,
npcap]` dans `publish-smoke.yml` — un écart de nom de label empêche le job
de trouver le runner.

Installer le runner en tant que **service Windows** (option proposée par
`config.cmd`/`svc install`) pour qu'il reste disponible sans session
utilisateur ouverte.

## 4. Vérifier

1. Déclencher manuellement le workflow **Publish Smoke** (`workflow_dispatch`
   depuis l'onglet Actions).
2. Vérifier que le job `windows-e2e-npcap` passe et que l'artefact
   `windows-e2e-npcap-logs` contient un `summary.txt` avec
   `SONAR_SMOKE_DEVICE=<nom d'interface réelle>`.
3. Relancer une deuxième fois : un second passage réussi confirme que le
   nettoyage (désinstallation) fonctionne et que la VM reste dans un état
   propre entre deux runs.

## Rappel sécurité

`publish-smoke.yml` doit rester `workflow_dispatch`-only. **Ne jamais**
ajouter de déclencheur `pull_request`/`pull_request_target` à ce workflow :
un runner self-hosted qui exécute du code venant d'une fork est un vecteur
d'exécution de code arbitraire sur une machine que le mainteneur contrôle.
