# Analyse complète de la codebase — 12/07/2026

Analyse réalisée par Claude (Fable 5) sur l'état de `main` à `3b561590`
(app 4.3.1, sonar-flows-core/cli 0.2.0, packet_parser 5.0.0). Elle succède à
l'audit externe du 11/07 (`11072026_gpt-5.6-sol ultra.md`) dont la campagne de
correctifs est intégrée : ce document constate l'état *après*, et cite les
issues ouvertes quand un constat est déjà suivi.

## 1. Vue d'ensemble

| Surface | Techno | Volume | Rôle |
|---|---|---|---|
| `src/` | Vue 3 + Pinia + Sigma, Deno | ~8 500 LOC | UI : graphe, matrice, panneaux, store d'événements |
| `src-tauri/` | Rust, Tauri 2 | ~7 600 LOC | Adaptateur desktop : capture live, commandes IPC, événements |
| `sonar-rust/crates/sonar-flows-core` | Rust pur, 0 dép. Tauri | ~3 200 LOC | Domaine : matrice, CSV, paquets/tunnels, graphe, import PCAP |
| `sonar-rust/crates/sonar-flows-cli` | clap | 74 LOC | MVP CLI : pcap → CSV, fusion de matrices |

L'architecture cible de VISION.md (« deux formes, un seul cœur ») est
**effective** depuis le 11-12/07 : `state/flow_matrix`, `state/graph` et
`messages/capture` du desktop sont des réexports purs du cœur, et les imports
CSV/PCAP délèguent au cœur (`d25eee9f`). Le vendor cargo est la source unique
des builds src-tauri (`.cargo/config.toml` → `vendored-sources`), régénéré à
chaque bump.

## 2. Forces

- **Séparation domaine/adaptateur réelle et testée** : 35 tests cœur + 6 CLI
  + 77 desktop, tous verts ; les sémantiques sensibles (fusion tunnel par
  tunnel, `origin`, troncature PCAP ≠ EOF, aller-retour CSV) sont testées sur
  fixtures réelles.
- **Pipeline de capture soigné** : machine d'état explicite avec `session_id`
  IPC (les événements périmés sont filtrés côté store front), pool de buffers
  borné avec promotion jumbo, batches paquets 256/75 ms, stats dédupliquées à
  250 ms, backpressure bornée (#141), drainage à l'arrêt, sérialisation
  WebView plafonnée (`PACKET_BATCH_UI_MAX = 16`).
- **Imports transactionnels** : matrice et graphe reconstruits en local,
  l'état partagé n'est remplacé qu'en cas de succès complet ; validation CSV
  stricte avec numéro de ligne ; exclusion mutuelle imports/capture.
- **Erreurs typées de bout en bout** : enums thiserror par domaine,
  sérialisation discriminée `{ kind, message }` consommée par
  `src/errors/capture.ts` ; la distinction ouverture/lecture PCAP survit
  jusqu'au front.
- **Supply chain au-dessus de la moyenne** : vendor commité, cargo
  audit/deny/udeps/outdated en CI, SBOM (deno.lock réel depuis #137), builds
  reproductibles outillés (`SOURCE_DATE_EPOCH`, `--remap-path-prefix`,
  `repro-env-check.yml`), licences AGPL-3.0-only alignées (#152), Npcap
  confiné aux bundles Windows.
- **CI trois surfaces** (#135) : fmt + clippy `-D warnings` + tests sur
  src-tauri **et** sonar-rust **et** front (vue-tsc + tests Deno), coverage
  (covecode), Trivy, SonarCloud.
- **Docs vivantes** : VISION.md, TUNNELS.md, todo.md tenu, sprints tracés,
  cahier de recette VAE.

## 3. Faiblesses et risques

### 3.1 Frontend — la zone la plus faible

- **Typage IPC de façade (#142)** : 44 `any` hors tests. Le cœur du problème
  est `store/capture.ts` : `onmessage = (msg: any)`, listeners `Array<(d:
  any) => void>`. Conséquence concrète : renommer un champ d'événement côté
  Rust ne fait échouer ni vue-tsc ni les tests — la régression n'apparaît
  qu'à l'exécution. Les types existent pourtant (`types/capture.ts`, 220
  lignes) mais sont écrits à la main, donc dérivables du Rust sans garantie.
- **Composants monolithes** : `ImportPanel.vue` 812 LOC (Options API, 22
  fonctions), `NetworkGraphComponent.vue` 805, `Filter.vue` 642. Peu
  testables unitairement, coût d'entrée élevé.
- **Code mort trompeur (#145, confirmé)** : `homeView.vue` (et ses enfants
  `homeVue/Capture.vue`, `FromPcap.vue`) n'est importé nulle part ;
  `Matrice.vue` (262 LOC) n'est importé nulle part ; `/readPcap` est routé
  mais orphelin. Un nouveau contributeur ne peut pas savoir ce qui est vivant.
- **Accessibilité inégale (#144)** : le pattern dialogue complet
  (`role="dialog"`, aria-modal, focus, Échap) n'existe que sur
  `LabelsPanel.vue` ; `ImportPanel`, `Filter`, `MatrixLabelsPanel`,
  `ConflictDialog` ont des overlays sans sémantique.
- **Tests minces** : 6 fichiers ciblés (bpf, captureStore, graphSync,
  labelImport, dateUtils, vues) pour 8 500 LOC ; les composants lourds n'ont
  aucun test ; pas d'E2E (#146). C'est le déséquilibre majeur du projet :
  le backend est très couvert, le front presque pas.
- 27 `console.*` hors tests (#112 partiellement traité : le chemin chaud
  paquet est propre, il reste des logs de cycle de vie).

### 3.2 Backend desktop

- **`commandes/import/labels.rs` : 1 138 LOC**, le plus gros fichier du
  projet — lecture, validation, normalisation, conflits, arbitrage dans un
  seul module. Fonctionne et est testé, mais c'est le prochain candidat
  naturel au découpage (la partie « domaine labels » reste desktop par
  décision du 12/07, ce qui n'empêche pas de la modulariser sur place).
- **Identité des nœuds du graphe par IP** (reliquat #154) : le multi-MAC est
  détecté et affiché comme anomalie, mais la labellisation reste indexée sur
  la première MAC ; la refonte de l'identité des nœuds est le seul morceau
  non traité de la sémantique graphe.
- **Double variante `capture_timing`** : `process_packet` /
  `process_packet_timed` (import) et l'équivalent capture live dupliquent le
  chemin nominal sous cfg. C'est le prix de l'instrumentation fine, mais les
  deux chemins peuvent diverger silencieusement (le précédent amont : l'effet
  `capture_timing` de packet_parser 4, corrigé en 5.0.0). Un test « les deux
  variantes produisent la même matrice » sur un pcap de référence fermerait
  le risque.

### 3.3 Cœur et CLI

- **CLI volontairement minimal** (74 LOC) : pas de labels, pas de filtre,
  codes de sortie et stderr corrects. La promesse « orchestration » de
  VISION.md repose donc entièrement sur #156 (arguments de session du
  desktop), pas encore entamé.
- **Publication crates.io** : 0.1.0 publiée, 0.2.0 commitée mais publication
  en attente d'authentification ; pas de CHANGELOG ni de politique de version
  documentée pour les crates (le CHANGELOG racine suit l'app desktop).
- Les structures `GraphData`/`GraphUpdate` du cœur sont sérialisées vers le
  front sans schéma partagé — même famille de risque que #142.

### 3.4 Tests et fixtures

- **Fixtures PCAP réelles absentes du repo (#151, confirmé)** : 1 test
  `ignored` visible, et les tests dépendant de `LOC42.pcapng` retournent
  silencieusement `Ok` quand le fichier manque. Le filet de sécurité du
  chemin tunnels dépend d'un fichier que la CI n'a pas.
- Un benchmark de pool existe (`examples/pool_bench.rs`) mais rien n'est
  suivi dans le temps ; la perf sous forte charge est un sprint ouvert non
  entamé (#132).

### 3.5 Process

- **#138 Npcap reste LA décision bloquante** avant tout tag Windows public
  (le technique est prêt : embarquement Windows-only, position écrite).
- **Issues de mai à trier** (#111, #112, #118-#121, #124) : plusieurs sont
  probablement obsolètes après les refontes de juillet (ex. #111 TopBar,
  #118 WiX/MSI après le déplacement de Npcap dans la conf Windows). Un
  passage de triage éviterait de traîner un backlog fantôme.

## 4. Recommandations priorisées

**P1 — verrouiller ce qui peut casser en silence**
1. Typage IPC (#142) : générer les types TS depuis Rust (`ts-rs` ou
   `specta`, qui s'intègre bien à Tauri) plutôt que de retaper
   `types/capture.ts` à la main ; typer `onmessage` et les listeners du
   store. C'est le meilleur ratio risque éliminé / effort du backlog.
2. Fermer #145 : supprimer `homeView`, `Matrice.vue`, trancher `/readPcap`.
   Une après-midi, et la carte du front redevient lisible.
3. Publier les crates 0.2.0 (dry-run déjà vert) et fermer #133.

**P2 — combler le déséquilibre de tests**
4. Un E2E smoke (#146) sur les deux parcours critiques : import matrice CSV
   → graphe, et capture sur `lo` → arrêt → export. Même un seul scénario
   WebDriver vaut mieux que zéro.
5. Fixture PCAP committable (#151) : rejouer un petit pcap anonymisé (le
   test CAPWAP du cœur montre que quelques trames suffisent) pour que la CI
   exécute réellement les tests tunnels.
6. Test d'équivalence des variantes `capture_timing` (même pcap → même
   matrice).

**P3 — dette structurelle, au fil de l'eau**
7. Découper `ImportPanel.vue` et généraliser le pattern dialogue de
   `LabelsPanel` (#144).
8. Découper `commandes/import/labels.rs` en sous-modules.
9. Refonte identité des nœuds (reliquat #154) — à cadrer avant de toucher.
10. Triage des issues de mai + décision Npcap (#138).

## 5. Verdict

Le backend (desktop + cœur) est dans un état remarquablement sain pour un
projet de cette taille : architecture voulue et réellement appliquée, dette
identifiée et tracée, CI qui mord, supply chain outillée. Le déséquilibre est
côté front : gros composants non testés, typage IPC de façade et code mort —
c'est là que les prochaines régressions arriveront si rien ne bouge, et c'est
là que le backlog (#142, #144, #145, #146) est déjà correctement pointé. Rien
de découvert dans cette analyse ne nécessite une nouvelle issue : le backlog
existant couvre les constats, ce document ajoute surtout la priorisation.
