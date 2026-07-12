La codebase est techniquement solide et nettement plus mature que la moyenne des applications Tauri de cette taille. L’architecture de capture, le cœur Rust partagé et la posture supply-chain sont de bonnes bases.

  En revanche, SONAR n’est pas encore prêt à garantir pleinement sa promesse de relevé OT « fidèle et vérifiable », ni à publier une release Windows publique sans réserve. Les blocages principaux sont :

  - la redistribution de Npcap ;
  - des paquets PCAP potentiellement ignorés sans diagnostic ;
  - plusieurs courses entre capture, import et reset ;
  - des désynchronisations matrice/graphe/labels ;
  - une validation SFMS insuffisante ;
  - une chaîne de release qui ne vérifie pas exactement le contenu final des installateurs.

  ## Architecture générale

  La version applicative est cohérente en 4.3.1 dans package.json:4, src-tauri/Cargo.toml:3 et src-tauri/tauri.conf.json:4.

  La base représente environ 20 000 lignes applicatives, hors vendoring :

  - src/ : Vue 3, Pinia, Vue Router, Sigma/Graphology et ForceAtlas2 ;
  - src-tauri/ : adaptateur desktop, capture libpcap, commandes IPC, état partagé ;
  - sonar-rust/ : cœur métier sonar-flows-core et CLI réutilisant ce cœur ;
  - security/, script/ci/, .github/ : reproductibilité, SBOM, signatures et releases ;
  - observability/ : instrumentation locale Loki/Grafana/Promtail ;
  - src-tauri/vendor/ : dépendances Rust vendored pour l’utilisation hors ligne.

  Le flux principal est bien découpé :

  libpcap
    → thread de capture
    → pool de buffers + canal Crossbeam borné
    → thread de parsing
    → FlowMatrix + GraphData
    → événements Tauri batchés
    → store Pinia
    → Vue + Sigma

  Les états globaux sont enregistrés dans src-tauri/src/lib.rs:95. Le domaine matrice/PCAP/graphe est déjà extrait dans sonar-flows-core, conformément à la vision « deux formes, un seul cœur » de VISION.md:25.

  ## Points forts

  - Le pipeline live utilise un canal borné, des compteurs de pertes, du batching et un pool lock-free paresseux à deux tailles (src-tauri/src/state/capture/capture_handle/threads/packet_buffer.rs:81).
  - Les événements sont associés à un session_id et les threads sont joints à l’arrêt, ce qui limite les captures fantômes et événements périmés (src-tauri/src/state/capture/capture_handle/mod.rs:37).
  - Le frontend filtre également les anciennes sessions (src/store/capture.ts:75).
  - L’import PCAP est transactionnel : la nouvelle matrice et le graphe sont construits localement puis remplacés uniquement après succès (src-tauri/src/commandes/import/pcap.rs:448).
  - Le traitement live borne la cardinalité à 250 000 flux, batch les mises à jour et limite ce qui est sérialisé vers la WebView (src-tauri/src/state/capture/capture_handle/threads/processing.rs:40).
  - Les erreurs Rust sont structurées et les lints interdisent les unwrap/expect évitables en production.
  - L’export travaille sur un snapshot et écrit via un fichier temporaire avec fsync (sonar-rust/crates/sonar-flows-core/src/matrix.rs:606).
  - La CSP est restrictive et le frontend n’utilise pas v-html (src-tauri/tauri.conf.json:12).
  - Les versions npm sont épinglées, les actions GitHub sont référencées par SHA, les crates desktop sont vendored et la release génère hashes, signatures détachées et attestations.
  - La documentation stratégique est bonne et assume clairement les enjeux OT, air-gap et supply-chain.

  ## Risques critiques et élevés

   Priorité    Constat                                                     Impact
  ━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   P0          Release Windows incompatible avec la décision Npcap         Risque juridique et release publique non conforme
  ──────────  ──────────────────────────────────────────────────────────  ─────────────────────────────────────────────────────────────────
   P0          DLT PCAP non contrôlé et erreurs de parsing silencieuses    Matrice incomplète sans alerte
  ──────────  ──────────────────────────────────────────────────────────  ─────────────────────────────────────────────────────────────────
   P0/P1       Absence de métadonnées d’audit dans l’export                Impossible de prouver les conditions et la complétude du relevé
  ──────────  ──────────────────────────────────────────────────────────  ─────────────────────────────────────────────────────────────────
   P1          Courses capture/import/reset                                Matrice et graphe pouvant diverger ou être écrasés
  ──────────  ──────────────────────────────────────────────────────────  ─────────────────────────────────────────────────────────────────
   P1          Désynchronisation labels/graphe                             UI différente des exports et de l’état backend
  ──────────  ──────────────────────────────────────────────────────────  ─────────────────────────────────────────────────────────────────
   P1          Validation SFMS et compteurs insuffisants                   Comptabilité tunnel corrompue ou overflow
  ──────────  ──────────────────────────────────────────────────────────  ─────────────────────────────────────────────────────────────────
   P1          Contenu des installateurs non réellement attesté            Binaire publié différent de celui testé
  ──────────  ──────────────────────────────────────────────────────────  ─────────────────────────────────────────────────────────────────
   P1          Capabilities Tauri trop larges                              Impact accru d’une compromission WebView

  ### 1. Release Windows à bloquer

  Le dépôt indique explicitement qu’aucune release Windows publique ne doit être publiée sans licence OEM, retrait de Npcap ou distribution restreinte (security/licences.md:29). Pourtant le workflow construit Windows puis rend automatiquement la release
  publique (.github/workflows/publish.yml:75, .github/workflows/publish.yml:303).

  L’installeur NSIS présente aussi un défaut fonctionnel : le hook PREINSTALL tente d’exécuter Npcap depuis $INSTDIR (src-tauri/windows/hooks.nsh:4), alors que ce hook s’exécute avant la copie des fichiers, comme le confirme la documentation Tauri
  vendored (src-tauri/vendor/tauri-utils/src/config.rs:909).

  Enfin, les bundles sont construits avec --no-sign : Cosign ne remplace pas Authenticode, ni la signature/notarisation macOS.

  ### 2. Fidélité PCAP insuffisamment garantie

  Le cœur ouvre les PCAP puis transmet directement packet.data à un parseur Ethernet sans vérifier get_datalink() (sonar-rust/crates/sonar-flows-core/src/pcap.rs:51). Les formats Linux cooked, RAW, loopback, radiotap ou 802.11 peuvent donc être mal
  interprétés ou rejetés.

  Le CLI comptabilise les erreurs de parsing, mais l’import desktop normal fait seulement return lorsqu’un paquet ne se parse pas (src-tauri/src/commandes/import/pcap.rs:148). L’utilisateur reçoit un nombre total de paquets et une matrice, sans savoir
  combien de paquets ont été ignorés.

  L’export ne contient que les lignes de flux (sonar-rust/crates/sonar-flows-core/src/matrix.rs:672). Il manque notamment :

  - interface, filtre BPF et DLT ;
  - début/fin de capture ;
  - paquets reçus, perdus et non parsés ;
  - version du parseur, version SONAR et commit ;
  - hash des PCAP sources ;
  - identifiant de session et paramètres de capture.

  Pour un produit visant l’audit OT, un manifeste de session signé ou un sidecar JSON est nécessaire.

  ### 3. Courses de concurrence

  ensure_idle_for vérifie que l’application est inactive, mais le verrou est immédiatement relâché avant l’import long (src-tauri/src/commandes/import/pcap.rs:430, src-tauri/src/commandes/import/matrix.rs:137). Une capture ou un second import peut
  démarrer entre la vérification et le commit.

  Le reset vide le graphe puis la matrice avec deux verrous distincts (src-tauri/src/commandes/net_capture.rs:142), alors que le processing met à jour la matrice puis le graphe. Un entrelacement peut laisser les deux incohérents.

  Au plafond de 250 000 flux, le processing quitte sans drainer le canal (src-tauri/src/state/capture/capture_handle/threads/processing.rs:648). Les paquets déjà acceptés sont alors perdus sans apparaître dans app_dropped.

  Il faut introduire un état backend Importing protégé par un garde RAII, interdire le reset pendant une opération et centraliser l’ordre des verrous.

  ### 4. Cohérence labels et graphe

  La suppression d’un label ne retire pas toujours le label du nœud : refresh_labels fait continue lorsque la résolution retourne None (sonar-rust/crates/sonar-flows-core/src/graph.rs:531).

  Côté frontend :

  - clear_label_store retourne des mises à jour graphe, mais elles sont ignorées (src/components/AnalyseView/panels/ImportPanel.vue:393) ;
  - l’arbitrage ne renvoie au parent qu’un événement générique et recharge seulement les tables (src/components/AnalyseView/panels/ArbitrationDialog.vue:83) ;
  - un snapshot peut être suivi d’anciennes mises à jour encore présentes dans la file requestAnimationFrame, car le reset ne vide ni _queue ni _raf (src/components/AnalyseView/NetworkGraphComponent.vue:98, src/components/AnalyseView/
    NetworkGraphComponent.vue:329).

  Il faut attacher une révision monotone aux snapshots/updates et purger la file lors d’un reset ou chargement complet.

  ### 5. Robustesse du format SFMS

  La validation des lignes ne contrôle que les IP et la date (sonar-rust/crates/sonar-flows-core/src/matrix.rs:706).

  La colonne encap_id :

  - transforme un compte invalide en zéro ;
  - accepte une somme tunnel supérieure au nombre de paquets ;
  - ne rejette pas les doublons ;
  - utilise plusieurs sommes/additions u64 non vérifiées (sonar-rust/crates/sonar-flows-core/src/matrix.rs:72, sonar-rust/crates/sonar-flows-core/src/matrix.rs:203).

  Les timestamps PCAP négatifs sont convertis en u64 sans contrôle (sonar-rust/crates/sonar-flows-core/src/matrix.rs:800).

  La protection contre les formules CSV est appliquée aux labels, mais pas systématiquement aux champs réimportables comme origin, MAC ou protocoles. Le format devrait avoir un schéma versionné et des invariants stricts.

  ### 6. Chaîne de release

  Le workflow construit une première fois le binaire reproductible et le smoke-teste, puis reconstruit séparément les bundles hors environnement reproductible (.github/workflows/publish.yml:114). Le binaire contenu dans l’installeur n’est donc pas
  nécessairement celui testé et attesté.

  Autres écarts :

  - les scans Trivy des artefacts interviennent après publication et ne bloquent pas la finalisation ;
  - aucune vérification n’aligne le tag avec les trois manifests et le changelog ;
  - le smoke test contourne largement Tauri et ne teste pas une vraie installation ;
  - sonar-cli, pourtant annoncée comme deuxième forme du produit, n’est pas publiée ;
  - les SBOM committés sont encore en version 4.2.0 ;
  - cargo-vet ne contient aucun audit local effectif et repose sur des centaines d’exemptions.

  ### 7. Surface Tauri

  La CSP est bonne, mais la capability principale accorde shell, process, OS, création de dossiers et écriture récursive sous $HOME (src-tauri/capabilities/default.json:8). Plusieurs de ces permissions/plugins ne semblent pas utilisés par le frontend
  actif.

  Le principe du moindre privilège n’est donc pas respecté malgré une bonne protection WebView.

  ## Frontend et performances

  Le contrat IPC TypeScript est largement désaligné avec Rust : champs camelCase contre snake_case, stats imbriquées contre payload plat, graphData contre graph_data (src/types/capture.ts:131). Le dispatcher utilise any, ce qui masque ces divergences
  (src/store/capture.ts:75).

  Autres problèmes notables :

  - le journal live s’abonne aux batches et aux paquets unitaires, alors que le store rediffuse chaque paquet d’un batch : travail doublé et doublons possibles (src/components/AnalyseView/BottomLong.vue:138) ;
  - le graphe rescane les arêtes incidentes à chaque mise à jour de taille (src/components/AnalyseView/graph/graphSync.ts:39) ;
  - ForceAtlas2 reste actif, et les arêtes parallèles sont recalculées globalement ;
  - les imports hors ligne n’ont pas de plafond de cardinalité ou d’annulation ;
  - l’export clone et trie la matrice sous mutex, ce qui peut provoquer des drops live ;
  - les listes de labels ne sont pas virtualisées ;
  - le filtre peut être masqué dans l’UI pendant qu’il reste réellement actif dans libpcap (src/components/NavBar/status-bar/StatusBar.vue:91) ;
  - les statistiques d’un import PCAP multi-fichier affichent seulement le total du dernier fichier ;
  - /readPcap est une route legacy cassée ;
  - accessibilité, focus des modales, i18n et métadonnées HTML restent incomplètes.

  La matrice et les labels utilisateur sont essentiellement en mémoire. La fermeture appelle directement exit(0) après confirmation (src/utils/appExit.ts:19), sans sauvegarde automatique, état « dirty » ou restauration de session.

  ## Tests et qualité vérifiés

  Tous les contrôles disponibles localement sont passés :

  - deno task test : 39/39 ;
  - deno task typecheck : réussi ;
  - deno task build : réussi ;
  - JS principal : 439,91 kB, 129,71 kB gzip ;
  - backend desktop : 77/77 tests Rust ;
  - cœur et CLI : 41/41 tests Rust ;
  - Clippy avec -D warnings : réussi sur les deux workspaces ;
  - cargo fmt --check : réussi sur les deux workspaces.

  Ces résultats doivent être relativisés :

  - les tests frontend utilisent --no-check (deno.json:8) ;
  - les tests TypeScript sont exclus de vue-tsc ;
  - deux tests tunnel « réels » retournent simplement lorsque LOC42.pcapng est absent (src-tauri/src/commandes/import/mod.rs:59) ;
  - la couverture CI frontend n’exécute essentiellement que les tests JavaScript racine ;
  - aucune couverture E2E WebView/Tauri, capture réelle, concurrence ou installation multi-OS ;
  - cargo audit et cargo deny n’étaient pas installés localement et n’ont donc pas été exécutés.

  ## Dette documentaire et dépôt

  - La vision et les dossiers supply-chain sont riches, mais plusieurs affirmations ne correspondent plus aux workflows.
  - Le README de sonar-rust utilise encore les anciens noms de crates.
  - Le README décrit mal les artefacts réellement publiés.
  - Le texte AGPL complet n’est pas inclus dans LICENSE.md.
  - observability/data/sonar-logs/sonar-timing.jsonl est un fichier généré committé de près de 50 Mo.
  - Grafana/Loki sont exposés sur toutes les interfaces avec des valeurs par défaut faibles dans le compose local.
  - Le vendoring desktop est cohérent avec l’air-gap, mais le workspace CLI possède une résolution et un lockfile séparés sans configuration de vendoring équivalente.

  ## Feuille de route recommandée

  1. Avant toute release : bloquer Windows en CI, trancher Npcap, corriger l’installeur, aligner tag/version, signer nativement et vérifier les bundles avant publication.
  2. Garantir la fidélité : détecter/refuser les DLT non supportés, compter les erreurs de parsing et produire un manifeste de session avec hashes, filtres, pertes et versions.
  3. Rendre l’état atomique : introduire Importing, sérialiser import/capture/reset, drainer le canal et rejeter les listes d’import vides.
  4. Fiabiliser SFMS : validation stricte de encap_id, opérations numériques sûres, timestamps validés, échappement CSV complet et schéma versionné.
  5. Réparer le graphe : suppression effective des labels, propagation systématique des updates et révisions snapshot/update.
  6. Aligner le contrat IPC : générer les types TypeScript depuis Rust et supprimer les any du dispatcher.
  7. Renforcer les tests : fixture PCAP synthétique versionnée, tests de concurrence, tests de contrat IPC, E2E Tauri et tests d’installateurs.
  8. Optimiser et nettoyer : import annulable/borné, indexation incrémentale du graphe, suppression du double fan-out paquet, persistance/recovery, suppression du legacy.

  Aucun fichier suivi n’a été modifié pendant l’analyse. La seule modification présente dans le worktree était déjà existante dans src/components/NavBar/status-bar/StatusBar.vue:1.