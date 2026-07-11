  Je ferais sonar-core comme une lib Rust pure, publiable sur crates.io, sans dépendance Tauri :

  sonar-core
    matrix       # FlowMatrix, FlowMatrixRow, merge, export CSV
    pcap         # PCAP/PCAPNG -> FlowMatrix
    labels       # import/export labels, conflits
    graph        # optionnel si considéré domaine métier
    errors       # thiserror, erreurs stables

  Puis :

  sonar-cli         # clap, stdout/stderr, exit codes
  sonar-desktop     # Tauri UI, invoke handlers, windows, events

  Le point important : sonar-core ne doit pas connaître tauri::State, Channel, AppHandle, WebView, menus, logs Tauri, etc. Il doit exposer des fonctions simples :

  let matrix = sonar_core::pcap::convert_files(&paths, options)?;
  sonar_core::csv::write_matrix(&matrix, output)?;

  Pour le desktop, les commandes Tauri deviennent juste des adaptateurs :

  - récupèrent les args depuis l’UI
  - appellent sonar-core
  - traduisent le résultat en events/snapshots UI

  Pour crates.io, je ferais attention à deux choses :

  - feature flags : pcap devrait probablement être une feature, car libpcap/Npcap rend l’installation plus lourde.
  - API stable : ne pas publier trop tôt une API interne brouillonne. D’abord extraire proprement, puis publier quand sonar-cli et sonar-desktop utilisent vraiment la crate.

  Plan pragmatique :

  1. Créer un workspace Cargo.
  2. Extraire FlowMatrix, FlowMatrixRow, CSV import/export dans sonar-core.
  3. Extraire PCAP -> matrix dans sonar-core, avec callback de progression optionnel.
  4. Adapter Tauri à la nouvelle lib.
  5. Créer sonar-cli avec clap.
  6. Publier sonar-core quand l’API est propre.

  Donc oui : sonar-core + sonar-cli + sonar_desktop_app, c’est beaucoup plus sain que d’étendre indéfiniment le headless Tauri.

---

# Reste à faire — audit de code du 10/07/2026

Points vérifiés et suivis en issues (le corrigé est déjà sur main :
capture fantôme, drainage à l'arrêt, import PCAP transactionnel, cycle
d'import frontend, bidir/multi-MAC du graphe, export de logs, vue-tsc,
typecheck, tests mockIPC).

## Sprint proposé : fiabiliser CI et release (project_management/sprint_ci_release_fiabilisation.md)

- [ ] [#135](https://github.com/Sonar-team/Sonar_desktop_app/issues/135) CI : workflows cassés et gates manquants (clippy, sonar-rust, typecheck, tests front, fmt, artefact macOS)
- [ ] [#136](https://github.com/Sonar-team/Sonar_desktop_app/issues/136) Release atomique et reproductibilité démontrable
- [x] [#137](https://github.com/Sonar-team/Sonar_desktop_app/issues/137) SBOM frontend invalide (deno.lock non catalogué par Syft)
- [ ] [#138](https://github.com/Sonar-team/Sonar_desktop_app/issues/138) Licence Npcap avant release Windows publique (décision, bloquant)

## Backlog robustesse pipeline

- [x] [#139](https://github.com/Sonar-team/Sonar_desktop_app/issues/139) Exclusion mutuelle capture/import/export (verrous tenus pendant l'I/O disque)
- [x] [#140](https://github.com/Sonar-team/Sonar_desktop_app/issues/140) Pool de buffers : famine des jumbo frames
- [x] [#141](https://github.com/Sonar-team/Sonar_desktop_app/issues/141) Télémétrie backpressure trop bavarde sous saturation

## Backlog qualité frontend

- [ ] [#142](https://github.com/Sonar-team/Sonar_desktop_app/issues/142) Typer les contrats IPC TypeScript (msg: any, snake/camel)
- [ ] [#144](https://github.com/Sonar-team/Sonar_desktop_app/issues/144) Accessibilité des modales
- [ ] [#145](https://github.com/Sonar-team/Sonar_desktop_app/issues/145) Vues/routes mortes ou cassées (/readPcap, homeView, Matrice.vue)

## Backlog transverse

- [ ] [#143](https://github.com/Sonar-team/Sonar_desktop_app/issues/143) Capacités Tauri : permissions mortes/dupliquées, bloc fs.scope ignoré
- [ ] [#146](https://github.com/Sonar-team/Sonar_desktop_app/issues/146) Stratégie E2E : capture réelle, installateurs Windows/macOS

# Reste à faire — audit externe du 11/07/2026 (analyses/11072026_gpt-5.6-sol ultra.md)

L'audit recoupe largement #135–#146 (compléments postés en commentaires sur
#135, #138 et #142) ; la dérive `while let Ok` de sonar-flows-core qu'il
signalait est corrigée. Points nouveaux tracés :

## Fiabilité en exploitation

- [x] [#147](https://github.com/Sonar-team/Sonar_desktop_app/issues/147) Mémoire non bornée en capture longue (matrice/graphe sans éviction, rétention logs illimitée)
- [x] [#149](https://github.com/Sonar-team/Sonar_desktop_app/issues/149) Machine d'état de capture et identifiant de session IPC
- [x] [#155](https://github.com/Sonar-team/Sonar_desktop_app/issues/155) Retirer le mode headless du desktop — sans GUI, c'est sonar-cli (décision VISION.md)
- [ ] [#156](https://github.com/Sonar-team/Sonar_desktop_app/issues/156) Arguments de session au lancement du desktop (interface, filtre, autostart, export) pour l'orchestration

## Fidélité des données

- [x] [#148](https://github.com/Sonar-team/Sonar_desktop_app/issues/148) Exports/imports CSV : déterminisme, écriture atomique, injection de formule, encap_id instable
- [ ] [#150](https://github.com/Sonar-team/Sonar_desktop_app/issues/150) Vérifier le DLT libpcap et neutraliser l'effet de capture_timing (packet_parser, correctif amont)
- [x] [#153](https://github.com/Sonar-team/Sonar_desktop_app/issues/153) Import de labels : chemin inexistant vidant le store, header heuristique, normalisation MAC/IP
- [x] [#154](https://github.com/Sonar-team/Sonar_desktop_app/issues/154) Graphe/stats : ports d'arête tronqués, processed mal libellé, stats effacées à l'arrêt

## Tests et conformité

- [ ] [#151](https://github.com/Sonar-team/Sonar_desktop_app/issues/151) Tests PCAP réels silencieusement sautés quand LOC42.pcapng est absent
- [x] [#152](https://github.com/Sonar-team/Sonar_desktop_app/issues/152) Licence : texte AGPL « or later » vs manifests AGPL-3.0-only (décision)
