# Feature : E2E Tauri et validation des installateurs

> Epic : distribution — Statut : planifié P0 (phase 3)
> Issue : [#146](https://github.com/Sonar-team/Sonar_desktop_app/issues/146)

Tests E2E Tauri des parcours capture/import/matrice/graphe/labels/export et
installation réellement testée sur chaque OS supporté (machine propre).

## User stories

- [ ] US-01 — à rédiger : chaque release prouve que les parcours principaux
  fonctionnent sur les OS supportés
  - Linux : fait — `.gitlab/ci/e2e-vm.yml` installe le `.deb` signé sur une
    VM et rejoue capture/import/matrice/graphe/labels/export
    (`script/e2e/run-sonar-x11-e2e.sh --live-capture`).
  - Windows : partiel (30/07/2026) — job `windows-e2e-npcap`
    (`.github/workflows/publish-smoke.yml`, `workflow_dispatch` uniquement,
    runner self-hosted) installe le NSIS de release et vérifie via
    `--sonar-smoke-test` que Npcap est détecté. La capture live et les
    autres parcours (import/matrice/graphe/labels/export) restent à faire
    sur Windows.
  - macOS : à faire.
- [ ] US-02 — intégrer Cypress Component et des E2E navigateur rapides pour
  Vue/Vite, avec un faux backend centralisé (`mockIPC`, événements et fenêtres
  Tauri), des sélecteurs stables et une exécution headless en CI
- [ ] US-03 — conserver une suite WebdriverIO/Tauri distincte pour valider le
  vrai binaire, l’IPC Rust, les plugins natifs et les différences de WebView
  sur Windows, Linux et macOS
