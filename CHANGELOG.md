# Changelog

## Non publié

## **[4.8.3] - 2026-07-27**

## ✨ Améliorations

- **Import direct des matrices Excel** : le sélecteur et le glisser-déposer
  acceptent désormais les fichiers `.xlsx`. La première feuille est lue avec
  le même schéma et la même validation stricte que les matrices CSV ; les
  colonnes supplémentaires, même insérées au milieu, sont ignorées tant que
  les en-têtes Sonar sont conservés. Les dates Excel sont normalisées au
  format SFMS et les imports mixtes CSV/XLSX conservent la provenance de
  chaque fichier.

## 🛠 Corrections

- **Arrêt fiable de la capture** (#166) : élimination d'un interblocage
  possible entre le démarrage et l'arrêt de la capture ; l'arrêt du pipeline
  est désormais fiable même en cas d'enchaînement rapide des deux actions.
- **Sessions préservées lors des imports rejetés** (#167) : une opération
  d'import refusée (fichier invalide, validation échouée) ne supprime plus
  les sessions existantes ; l'état de travail est restauré à l'identique.
- **PCAPNG multi-interfaces** : l'import d'un fichier PCAPNG mêlant plusieurs
  types d'interfaces affiche désormais un message d'erreur actionnable au
  lieu d'une erreur générique.
- **Barre de statut** : suppression du compteur redondant des paquets intégrés
  à la matrice ; les pertes et erreurs de parsing restent affichées pour
  expliquer les écarts avec les trames reçues. Les compteurs restent lisibles
  au-delà de 1 000 sans chevauchement.

## **[4.8.2] - 2026-07-22**

## 🛠 Corrections

- **CI de l'oracle PCAP restaurée** : le script ShellCheck utilise désormais
  une affectation portable de `CDPATH`, la configuration Dependabot est valide
  et le changelog expose la version attendue par les contrôles de publication.

## **[4.8.1] - 2026-07-22**

## ✅ Tests

- **Exactitude des matrices PCAP contrôlée avec TShark** (#168) : les captures
  Ethernet, Linux cooked SLL/SLL2 et CAPWAP de la crate `packet_parser` sont
  comparées flux par flux à des oracles TShark versionnés. Les champs communs,
  l'encapsulation, le refus des captures non prises en charge et la stabilité
  octet pour octet des exports SFMS sont vérifiés automatiquement par la CI.

## **[4.7.0] - 2026-07-16**

## 🛠 Corrections

- **Gravité du graphe sous Windows** : ForceAtlas2 peut démarrer son Web
  Worker dans WebView2 grâce à la CSP dédiée aux workers locaux `blob:`.
- **Import des labels Forge** : les en-têtes `adresse_mac,adresse_ip,label`
  sont reconnus sans relâcher la validation de la première ligne.

## **[4.6.2] - 2026-07-16**

## 🛠 Corrections

- **Build de release Linux à nouveau reproductible entre clones isolés** :
  `sonar-flows-core 0.3.0` est publié, référencé avec une version exacte puis
  vendorisé dans l'application. Cargo utilise ainsi une identité de source
  stable au lieu du chemin local du checkout, qui faisait diverger les
  binaires et bloquait la publication après comparaison des SHA-256.
- **Publication des SBOM restaurée** : le générateur du SBOM frontend est
  désormais suivi comme exécutable. Le workflow peut générer, téléverser,
  attester et signer les SBOM backend et frontend au lieu d'échouer avec le
  code 126.

## 🔧 Maintenance

- Les crates `sonar-flows-core` et `sonar-flows-cli` passent en version
  `0.3.0`. Le cœur partagé est inclus dans le vendor hors ligne de
  l'application et son inventaire de licences est mis à jour.

## **[4.6.1] - 2026-07-16**

## ✨ Améliorations

- **Rapport qualité d'import visible** (#150) : l'événement `Finished` de
  chaque fichier importé porte désormais sa comptabilité complète — paquets
  lus, intégrés à la matrice, illisibles par le parseur — et la barre de
  statut affiche les illisibles (🧩) après un import comme pendant une
  capture. Les paquets illisibles sont maintenant comptés dans l'import
  desktop (ils étaient perdus hors instrumentation).
- **Réimport RAW/SLL/SLL2 sans éclater les fusions multi-sondes** (#150) :
  l'aller-retour CSV conserve exactement l'identité SFMS. Pour Linux cooked,
  seuls l'adresse source et le protocole identifient la conversation ;
  direction, type matériel, longueur déclarée, champ réservé et index
  d'interface restent des métadonnées fidèles du paquet observé mais ne
  segmentent plus la matrice. Aucune colonne `link_details` n'est ajoutée,
  et `origin` cumule les fichiers ayant vu le flux. Les captures réelles SLL
  (2 702 trames) et SLL2 (779 trames) couvrent le round-trip bout en bout.
- **Le contrat IPC complet est généré depuis Rust, versionné, et son
  exhaustivité Rust → miroir est imposée par le compilateur** (#142, ts-rs) :
  `src/types/generated/` est écrit par `cargo test export_ipc_bindings` et la
  CI échoue si le contrat commité a dérivé des types Rust — la gate compare
  désormais `git status --porcelain` après suppression et régénération
  complète du dossier (`git diff` seul laissait passer un fichier généré non
  suivi par git, ce qui était le cas des 18 premiers fichiers de cette
  fonctionnalité). Les cinq familles d'erreurs, le payload `Stats` et l'union
  complète `CaptureEvent` (graphe, batches de paquets,
  `started`/`stopped`/`finished`/`channelCapacityPayload`) ne peuvent plus
  mentir au frontend, qui consomme le contrat généré (`src/types/capture.ts`)
  au lieu de types dupliqués à la main. `crate::events::contract::to_contract`
  convertit chaque `CaptureEvent` réel en son miroir par un `match` sans bras
  `_` : ajouter une variante côté Rust sans mettre à jour le miroir casse la
  compilation, pas seulement un test qu'on pourrait oublier d'écrire. Les
  champs réellement omissibles (VLAN absent, couche applicative absente,
  paquet non tunnelé…) sont déclarés `#[ts(optional)]`/`#[ts(optional =
  nullable)]` : un paquet valide sans ces couches satisfait maintenant son
  propre type TypeScript, ce qui n'était pas le cas avant correction (le
  sérialiseur omettait la clé, le type généré l'exigeait). `Started` porte
  `protocol_version` (`CAPTURE_EVENT_PROTOCOL_VERSION`), vérifié par le store
  frontend et désormais émis par les trois chemins de session (capture live,
  import PCAP, **et import de matrice CSV**, qui commençait directement par
  `GraphSnapshot` sans jamais annoncer de session). Un tag d'événement inconnu
  est journalisé plutôt que de faire planter le store — cohérent avec la
  politique de version qui n'exige pas de bump pour une nouvelle variante. Le
  store (`src/store/capture.ts`) et la chaîne de rendu du graphe
  (`graphSync.ts`, `NetworkGraphComponent.vue`) n'ont plus de `any` sur les
  abonnements/payloads de capture, et le dispatcher est exhaustif (`never`
  côté TypeScript). La variante `Packet` (jamais construite côté Rust) est
  supprimée des deux côtés plutôt que mal mirée avec un champ `encap_id`
  inexistant sur le vrai type. 18 tests Rust vérifient, variante par variante
  — Ethernet/RAW/SLL/SLL2/IEEE 802.11, paquet tunnelé, couche corrompue,
  groupes internet/transport/application tous absents compris — que chaque
  miroir sérialise en JSON identique au vrai `CaptureEvent`. Bug annexe
  corrigé : `BottomLong.vue` s'abonnait à la fois à `onPacket` et
  `onPacketBatch`, doublant chaque trame affichée dans le tableau de capture.
- **#142, deuxième revue — 6 défauts de fidélité trouvés et corrigés** :
  `#[ts(optional)]` mal ciblé au premier passage (déjà listé ci-dessus) ;
  `encap_id` (paquet) sérialisait en `number` alors que c'est un hash 64
  bits qui peut dépasser `Number.MAX_SAFE_INTEGER` — sérialisé en hex 16
  caractères comme `Edge::encap_ids`, même convention ; l'import de labels
  (`import_label_file`) n'émettait ni `Started` ni version ; `BottomLong.vue`
  recastait le contrat généré vers un type `unknown` local au lieu de
  discriminer sur `link_kind` ; `normalizeGraphUpdate`, `nodeAttributes`,
  `edgeAttributes`, les reducers Sigma et le switch `applyUpdate` restaient
  en `any` ou sans garde `never` ; **aucun test ne validait le JSON contre
  TypeScript** — `src/types/captureContractFixtures.ts` assigne un littéral
  par variante à `CaptureEvent`, vérifié par `deno task typecheck` (CI).
  `cargo test export_bindings` (commentaire, `Cargo.toml`) corrigé en
  `export_ipc_bindings` (le premier ne matchait aucun test). Contrat
  d'erreurs : 5 tests de fidélité JSON par domaine ajoutés (`errors/mod.rs`),
  moins de risque de dérive que les événements car il n'y a pas de miroir
  dupliqué — `CaptureStateErrorKind` sert à la fois de cible de
  sérialisation et de type `ts-rs`.
- **`deno task typecheck` ne détectait plus aucune erreur** (trouvé pendant
  la revue de #142) : `deno_task_shell` avale silencieusement la sortie et
  le code de sortie du binaire `vue-tsc` résolu depuis `node_modules/.bin`
  quand il est invoqué par son nom nu — la tâche rendait « 0 erreur » même
  sur un fichier ne compilant pas. `deno.json` invoque désormais
  `node node_modules/.bin/vue-tsc --noEmit`, qui restaure la détection
  (reproduit et vérifié : erreur introduite délibérément, détectée après le
  correctif, invisible avant). Le typecheck frontend en CI (`rust-ci.yml`,
  job « Gates frontend ») appelle ce même `deno task typecheck` : il ne
  contrôlait donc jamais réellement le typage du frontend jusqu'ici.
- **#142, preuve Rust → JSON → TypeScript sur du JSON réellement produit
  par Rust** : `cargo test export_ipc_fixtures`
  (`src-tauri/src/events/contract.rs`) écrit
  `src/types/generated/captureEventFixtures.ts`, un JSON réel par variante
  et par nullabilité (Ethernet+VLAN, minimal, tunnelé avec `encapId` non
  nul, corrompu, SLL/SLL2/RAW/802.11 avec et sans adresse, groupes
  internet/transport présents mais champs individuels absents, nœud sans
  label, arête sans port), typé avec `satisfies CaptureEvent` — garde les
  tags en types littéraux sans figer les tableaux en `readonly` (`as
  const` cassait `macs`/`packets`/`updates`, mutables côté contrat) TOUT EN
  détectant les champs excédentaires qu'une inférence générique (`fx<T
  extends CaptureEvent>(v: T): T`, approche initiale, abandonnée) laissait
  passer silencieusement. `captureContractFixtures.ts` les assigne à
  `CaptureEvent` : vérifié que le regression exact rapporté (`encap_id:
  string` redéclaré `number`) est détecté par `deno task typecheck`. Une
  garde Rust complémentaire (`assert_no_phantom_contract_fields`) vérifie
  qu'aucun champ optionnel déclaré côté contrat ne reste systématiquement
  absent de tous les fixtures — un champ ajouté à la main sans provenir
  d'aucune donnée réelle resterait sinon invisible.
  `normalizeGraphUpdate`, `nodeAttributes`/`edgeAttributes`, le reducer
  `refreshParallelEdges` et `drawNodeLabel` n'ont plus de `any` (types
  Sigma/graphology dédiés) ; `StatusBar.vue` passe en `lang="ts"` (un bug
  mort trouvé au passage : `this.matrice_len` n'existait sur aucune
  déclaration, supprimé) ; l'import de labels vérifie aussi
  `protocol_version` (`LabelsPanel.vue`, constante partagée avec le store).
- Les types de liaison supportés et leurs limites sont documentés dans le
  README (#150).

## 🔧 Maintenance

- Tests des chemins « difficiles » (#88) : PCAP, matrices et labels sous des
  noms contenant espaces, accents, CJK, emoji, apostrophes, guillemets et
  backticks — aucun chemin ne passe par un shell. Limite documentée et figée
  par un test : un `|` dans un nom de fichier est découpé par la colonne
  `origin` (séparateur multi-fichiers).
- **Plus aucun test sauté silencieusement** (#151) : les tests de tunnels
  utilisent désormais `ndpi_capwap.pcap` (corpus public nDPI, 422 paquets,
  canal data CAPWAP avec trafic client DHCP/mDNS/ICMPv6), versionné dans le
  dépôt — la capture de mission LOC42.pcapng, non publiable, faisait passer
  ces tests pour verts quand elle était absente (CI). Le PCAPNG
  multi-interfaces (DLT mélangés, snaplens divergents) est forgé en dur dans
  les tests : rejet explicite figé, comme observé sur fichiers réels.
- Preuve de terminaison des imports (#87) : un PCAP pathologique (100
  enregistrements de longueur nulle) se termine avec chaque paquet classé ;
  un fichier tronqué ou d'un DLT inconnu échoue explicitement — le backend
  ne peut pas boucler. Le symptôme « import infini » restant plausible est
  l'UI qui reste verrouillée (`isConverting`), suivi dans #161.

## 🛠 Corrections

- **L'identité d'une ligne de tunnel n'inclut plus la conversation
  encapsulée** : un même tunnel produisait une ligne externe par conversation
  interne (six lignes CAPWAP au 5-uplet identique sur la capture nDPI), une
  distinction que l'export CSV ne sait pas exprimer — l'aller-retour
  export → réimport fusionnait ces lignes et n'était donc pas inversible.
  La ligne externe agrège désormais par flux externe ; les conversations
  internes restent des lignes à part entière, jointes par `encap_id`.
- L'import (PCAP ou matrice CSV) réserve désormais **atomiquement** la phase
  `Importing` pendant toute la conversion, swap final inclus (#139) : un
  démarrage de capture pendant un import est refusé par la machine d'état au
  lieu de créer un relevé hybride silencieusement écrasé en fin d'import. La
  réservation est rendue sur tous les chemins de sortie (succès, erreur,
  panique).
- **Aucun paquet accepté n'est plus perdu au plafond de flux** (#158) : à
  l'arrêt sur plafond (#147), les paquets déjà acceptés dans le canal sont
  drainés vers la matrice comme à l'arrêt demandé, au lieu d'être détruits
  sans être comptés (dépassement borné par la taille du canal).
- **La comptabilité de capture boucle** (#158) : les paquets illisibles par
  le parseur et les paquets intégrés à la matrice sont comptés et exposés
  (`parse_errors` 🧩 et `integrated` dans l'événement `Stats`, illisibles
  affichés dans la barre de statut), et un **récapitulatif final** est émis
  après le drainage — les derniers compteurs affichés incluent les paquets
  drainés et un ultime relevé des stats pcap. Chaque paquet reçu appartient
  à une catégorie : intégré, illisible, ou perdu (noyau / interface /
  application).

## **[4.5.0] - 2026-07-14**

## ✨ Améliorations

- **Le type de liaison (DLT) réel est enfin respecté de bout en bout** (#150) :
  la capture live et l'import PCAP parsent chaque paquet avec le décodeur de
  son LINKTYPE (Ethernet, RAW IP, Linux cooked SLL/SLL2) au lieu de supposer
  Ethernet. Un DLT sans décodeur est refusé explicitement **avant** toute
  mutation du relevé — au démarrage de capture (avant l'événement `Started`)
  comme à l'import.
- **Préambule SFMS** : les matrices exportées commencent par une ligne
  `#SFMS version=1 dlt=…` qui porte les métadonnées du relevé. Un export
  antérieur (sans préambule) se réimporte inchangé, avec un DLT implicite
  Ethernet. Pas de date d'export dans le préambule : deux exports d'une même
  matrice restent identiques octet pour octet (déterminisme, #148).
- **La fusion inter-DLT est refusée partout** (arbitrage du 14/07/2026 : une
  fusion ne concerne que des relevés du même réseau, donc du même type de
  liaison) : fusion de matrices CSV, conversion PCAP multi-fichiers, et
  démarrage d'une capture sur une interface d'un autre DLT que le relevé en
  cours. Le réimport d'une matrice SLL est refusé explicitement tant que la
  reconstruction exacte n'est pas disponible, plutôt que dégradé en Ethernet.

## 🛠 Corrections

- Les erreurs d'import (`openFileError`, `readPacketError`), d'export et
  Tauri affichent à nouveau leur vrai message : les types TypeScript de
  `capture.ts` sont réalignés sur la sérialisation réelle des enums Rust
  (fichier affiché `undefined`, erreurs d'export en « Erreur inconnue », #142).

## 🔧 Maintenance

- Mise à jour de `packet_parser` en **8.1.0** : point d'entrée
  `parse(link_type, bytes)` multi-LINKTYPE et constructeurs publics owned
  SLL/SLL2 (préparation du réimport SLL exact).
- Corpus de test : captures réelles Linux cooked v1 (`sll.pcap`, 2702 trames)
  et v2 (`capture_sll2.pcap`, 779 trames) intégrées aux fixtures, avec tests
  bout-en-bout (conversion, comptabilité exhaustive des paquets, préambule,
  refus de réimport) qui échouent si la fixture manque (#151).

## **[4.4.0] - 2026-07-13**

## 🔒 Sécurité

- Retrait de l'installeur Npcap du dépôt et des bundles Windows : Npcap doit
  désormais être téléchargé séparément depuis son site officiel.
- Le bundle Windows est temporairement limité à NSIS. Son hook distingue Npcap
  absent, installé sans mode compatible et prêt pour SONAR, puis propose
  d'ouvrir la page de téléchargement officielle sans exécuter de binaire tiers.
- La CI refuse tout installeur Npcap/WinPcap dans le dépôt ou le bundle. Le
  smoke Windows devient structurel tant que l'usage de Npcap sur les runners
  n'est pas clarifié avec Nmap dans l'issue #138, qui reste ouverte.

## 🔧 Maintenance

- Mise à jour de `packet_parser` en **6.0.0** : durcissement de la détection
  MQTT (validation stricte MQTT 3.1/3.1.1, zéro faux positif sur le corpus de
  pcaps de la crate). Le workspace `sonar-rust` est aligné sur la même version
  pour éviter les doubles majeures dans le graphe de dépendances.
- Mise à jour de la version de SONAR en **4.4.0**.

## **[4.3.1] - 2026-07-09**

## 🛠 Corrections

- Élimination de tous les `unwrap()`/`expect()` du code de production : un
  mutex empoisonné, un fichier de log sans nom ou un canal d'événement
  indisponible remontent désormais une erreur structurée au frontend au lieu
  de faire paniquer le backend.

## 🔧 Maintenance

- Ajout des lints clippy `unwrap_used`/`expect_used` (bloquants en CI) pour
  empêcher la réintroduction de panics évitables ; les tests restent exemptés
  via `clippy.toml`.
- Référencement des issues GitHub de suivi dans les documents de sprint
  (#132 performance capture, #133 extraction sonar-core/cli).
- Mise à jour de la version de SONAR en **4.3.1**.

## **[4.3.0] - 2026-07-09**

## ✨ Améliorations

- Nouvelle colonne `origin` dans la matrice de flux : à l'import de plusieurs
  matrices CSV, chaque ligne indique le ou les fichiers dont elle provient.
  Quand un même flux est présent dans plusieurs fichiers, ses noms sont
  fusionnés (triés, joints par `|`, ex. `site-a.csv|site-b.csv`).
- Une matrice déjà fusionnée puis réimportée conserve la provenance de sa
  colonne `origin` : le nom du fichier de fusion n'est pas ajouté, seules les
  origines des colonnes sont réunies. Les flux issus d'une capture live ou
  d'un import PCAP ont une origine vide. La colonne est optionnelle à la
  lecture (compatibilité avec les matrices exportées avant cette version).

## 🔧 Maintenance

- Mise à jour de la version de SONAR en **4.3.0**.

## **[4.2.0] - 2026-07-08**

## ✨ Améliorations

- Surbrillance de la parenté des tunnels dans le graphe : survoler une arête
  illumine toute la famille du tunnel (ligne externe CAPWAP et flux internes,
  dans les deux sens) et estompe le reste du graphe ; un clic sur l'arête
  épingle la surbrillance, re-clic ou clic sur le fond la libère.
- La colonne `encap_id` de la matrice ventile désormais les paquets par tunnel
  (formes `id` ou `id1:n|id2:n|…`) : la somme des paquets attribués aux lignes
  internes d'un tunnel est exactement égale au compteur de sa ligne externe,
  tout en gardant une seule ligne par flux. L'import CSV comprend les trois
  formes (aller-retour export → import sans perte).
- Documentation du modèle père/fils des tunnels dans `TUNNELS.md` (vision,
  invariants, choix techniques, format SFMS de la colonne `encap_id`).

## 🛠 Corrections

- L'identifiant de tunnel est un hash de la paire d'extrémités, indépendant du
  sens : l'aller et le retour d'un même tunnel partagent le même `encap_id`.
  Plus de tunnels orphelins dans la matrice (sur la capture de référence :
  80 tunnels sur 80 équilibrés, contre 73 orphelins sur 84 auparavant).
- Les paquets de tunnel sans contenu décapsulable (keepalives CAPWAP) ne sont
  plus comptés dans la ligne externe du tunnel, préservant l'égalité
  père/fils.
- En développement, l'enregistrement d'une matrice dans
  `src-tauri/test_files/` ne fait plus redémarrer `tauri dev`
  (`.taurignore`).

## 🔧 Maintenance

- Mise à jour de la version de SONAR en **4.2.0**.

## **[4.1.1] - 2026-07-07**

## ✨ Améliorations

- Ajout de l'export des labels au format CSV réimportable `mac, ip, label`.
- Ajout d'une vue des labels réellement appliqués à la matrice, avec recherche
  par MAC, IP ou libellé.
- Ajout d'un module d'arbitrage des conflits de labels : l'import garde le
  premier label par clé `(mac, ip)` et laisse l'utilisateur choisir le bon label
  en cas de doublon.

## 🛠 Corrections

- L'import de labels ne bloque plus sur les doublons non destructifs et ignore
  les faux conflits liés aux IP placeholder ou aux MAC broadcast/multicast.
- L'export de labels complète les champs MAC/IP manquants à partir de la
  matrice de flux et évite de produire plusieurs labels contradictoires pour le
  même endpoint.

## 🔧 Maintenance

- Régénération des SBOM CycloneDX backend et frontend pour SONAR **4.1.1**.
- Mise à jour de la version de SONAR en **4.1.1**.

## **[4.1.0] - 2026-07-07**

## ✨ Améliorations

- Ajout de l'import multiple de matrices CSV : plusieurs matrices exportées
  peuvent être sélectionnées puis fusionnées en une seule analyse.
- Fusion automatique des flux identiques lors de l'import de matrices :
  cumul du nombre de paquets, cumul des octets et conservation de la date
  `last_seen` la plus récente.
- Passage du compteur d'octets de la matrice en `u64` afin de mieux supporter
  les fusions volumineuses.
- Amélioration du panneau d'import de matrices : les fichiers sélectionnés sont
  listés avant ouverture et peuvent être effacés avant import.
- Ajout d'un menu `A propos` avec accès séparé aux informations de version et
  au changelog embarqué.
- Affichage de la version Npcap utilisée dans les informations de version.

## 🛠 Corrections

- Amélioration des erreurs d'import de labels CSV : numéro de ligne, ligne
  complète et compteur total d'erreurs affichés pour faciliter la correction
  des fichiers.
- Tolérance accrue de l'import de labels CSV pour les colonnes supplémentaires
  et les colonnes finales vides.

## 🔧 Maintenance

- Ajout de `NPCAP_VERSION` dans les versions de build contrôlées par la CI.
- Mise à jour de la version de SONAR en **4.1.0**.

## **[4.0.1] - 2026-07-06**

## 🔧 Maintenance

- Mise à jour de Npcap embarqué en **1.88** pour les bundles Windows et le
  smoke runtime Windows.
- Ajout de la méthodologie de build sécurisé Tauri dans la documentation de
  gestion de projet.
- Mise à jour de la version de SONAR en **4.0.1**.

## **[4.0.0] - 2026-07-05**

## 💥 Changements majeurs

- Mise à jour de `packet_parser` en **2.0.2** : structure des paquets parsés
  typée (MAC, Ethertype, protocoles en `&'static str`), détection PostgreSQL
  par heuristique de payload et nouveaux parseurs applicatifs. La sérialisation
  vers le frontend reste compatible.
- Refonte du moteur de capture : nouveaux événements IPC `graphBatch` et
  `stopped`, nouveau champ `app_dropped` dans l'événement `stats`.

## ✨ Améliorations

- Pool de buffers à allocation paresseuse avec deux classes de tailles
  (2 KiB standard / snaplen jumbo) : mémoire de capture réduite de ~640 Mio
  préalloués à quelques Mio suivant la charge réelle (bench : RSS 464 → 13 Mio,
  débit ×2,7).
- Un seul verrouillage de la matrice de flux par paquet (labels + update dans
  le même scope) et états Tauri résolus hors de la boucle de traitement.
- Updates graphe coalescées par nœud/arête et envoyées par lot au rythme du
  batch de paquets, au lieu d'un événement IPC par paquet.
- Le flux n'est cloné dans la matrice qu'à sa première apparition, plus à
  chaque paquet.
- Stats pcap sorties du canal de données (état partagé atomique) : elles
  restent fiables sous backpressure.
- Pertes applicatives comptées (pool épuisé / canal plein), affichées dans la
  barre de statut (⚠️) et logs de perte agrégés à 1/s.
- Événement `stopped` émis sur erreur pcap fatale et à l'arrêt ; `stop()`
  attend la fin des threads pour éviter deux pipelines lors d'un redémarrage
  rapide.
- Résumé de run `capture_timing` enrichi des compteurs du pool
  (`pool_small_allocated`, `pool_large_allocated`, `pool_allocated_bytes`,
  `pool_exhausted`) et benchmark reproductible `examples/pool_bench.rs`.

## 🛠 Corrections

- Les octets du premier paquet de chaque flux n'étaient comptés qu'une fois
  au lieu de deux dans la matrice et le CSV exporté.
- Compilation `--features capture_timing` réparée (import `PathBuf` dupliqué).
- Artefacts de build des crates vendorées retirés du suivi git.

## 🔧 Maintenance

- Suppression du thread de traitement CLI dupliqué (~200 lignes) : le mode
  headless partage le pipeline principal et son instrumentation.
- Mise à jour de la version de SONAR en **4.0.0**.

## **[3.14.7] - 2026-07-03**

## 🛠 Corrections

- Fiabilisation de l'import des labels CSV : l'en-tête est écarté du store,
  les fichiers vides ne provoquent plus d'accès invalide, les IP CIDR sont
  normalisées et les labels absents utilisent la valeur `Label?`.
- Ajout d'un test d'import complet sur des fichiers CSV réels couvrant les
  résolutions par MAC/IP, IP seule, MAC seule, CIDR et label vide.

## 🔧 Maintenance

- Mise à jour de la version de SONAR en **3.14.7** pour publier le correctif
  d'import des labels.

## **[3.14.6] - 2026-06-29**

## 📊 Observabilité

- Ajout d'un jeu de mesures `sonar-timing.jsonl` pour les tests de performance
  capture, avec timings IPC par batch, timings détaillés du pipeline de parsing
  et résumés de runs.
- Ajout de données de référence pour comparer les runs `batch256`,
  `batch256-75ms` et `parser154` sur un volume d'environ 1,09 million de
  paquets.

## 🔧 Maintenance

- Mise à jour de la version de SONAR en **3.14.6** pour publier ces données
  d'observabilité.

## **[3.14.0] - 2026-06-24**

## ✨ Améliorations

- Mise à jour de `packet_parser` en **1.5.0**.
- Ajout du parsing **SNMP v1/v2c/v3** avec détection UDP 161/162, PDU
  standards et varbinds.
- Ajout du parsing **EtherNet/IP encapsulation**, détecté sans dépendance au
  port.
- Ajout d'une boîte de dialogue de conflits pour l'import de labels, avec
  affichage des conflits IP/MAC, IP/label, des formats MAC/IP invalides et des
  fichiers non importés.

## 🛠 Corrections

- Renforcement des validations du parser sur les couches data link, internet,
  transport et application.
- Remplacement progressif des erreurs non typées du parser par des erreurs
  dédiées par protocole.
- Migration de plusieurs parseurs vers une interface `TryFrom<&[u8]>` plus
  cohérente pour les usages temps réel.

## 🔧 Maintenance

- Réorganisation des validations et erreurs du parser dans des modules dédiés
  `checks` et `errors`.
- Ajout de documentation interne pour la méthode d'ajout de protocole dans
  `packet_parser`.
- Ajout du changelog interne de `packet_parser` et alignement du vendor sur la
  version **1.5.0**.
- Mise à jour de la version de SONAR en **3.14.0** pour publier ces changements.

## **[3.13.25] - 2026-06-23**

## ⚡ Performances

- Capture live : regroupement des événements paquets côté IPC pour réduire la
  pression sur le frontend lors des captures à fort débit.
- Bottom log : bufferisation non réactive des paquets et rafraîchissement limité
  à 10 fois par seconde, afin d'éviter les ralentissements autour de 1000
  paquets/seconde.

## 🛠 Corrections

- Bottom log : pré-formatage des lignes affichées et limitation stricte aux 5
  dernières trames visibles.
- Store capture : ajout d'un désabonnement effectif aux callbacks `onPacket`
  pour éviter l'accumulation de listeners après démontage/remontage du composant.
- Documentation : ajout de la configuration minimale recommandée dans le README.
- Mise à jour de la version de SONAR en **3.13.25** pour publier ces correctifs.

## **[3.13.24] - 2026-06-22**

## ✨ Améliorations

- Refonte complète de l'UX du panneau filtre BPF : nouvelle interface plus claire
  avec affichage de l'état du filtre actif, badge "En attente" (orange) lorsqu'un
  filtre est appliqué pendant une capture, et bouton "Annuler" pour revenir au
  filtre précédent.

## 🛠 Corrections

- Panneau filtre BPF : les presets rapides ne réinitialisent plus le filtre
  backend actif ; seul le formulaire local est remis à zéro avant d'appliquer le
  preset.
- Panneau filtre BPF : appliquer un filtre pendant une capture active le marque
  désormais comme « en attente » plutôt que « actif » ; il passe en actif
  automatiquement au prochain démarrage de capture.
- Thread de capture : suppression du `println!("TimeoutExpired")` parasite qui
  polluait la console lors de chaque tick pcap sans paquet.

## 🔧 Maintenance

- Mise à jour de Rust de 1.95.0 vers 1.96.0.
- Mise à jour de toutes les dépendances Rust (vendor regenerated).
- Mise à jour de toutes les dépendances frontend (Vite 8.0.9 → 8.0.16, Vue,
  Tauri plugins, etc.).
- Alignement des versions `.gitlab-ci.yml` avec `build-versions.env`.

## **[3.13.23] - 2026-06-21**

## 🛠 Corrections

- Panneau filtre BPF : les presets rapides ne réinitialisent plus le filtre
  backend actif ; seul le formulaire local est remis à zéro avant d'appliquer le
  preset.
- Panneau filtre BPF : appliquer un filtre pendant une capture active le marque
  désormais comme « en attente » (badge orange « Prochain démarrage ») plutôt que
  « actif » ; il passe en actif automatiquement au prochain démarrage de capture.
  Un bouton « Annuler » permet de revenir au filtre actif précédent.
- Thread de capture : suppression du `println!("TimeoutExpired")` parasite qui
  polluait la console lors de chaque tick pcap sans paquet.

## **[3.13.22] - 2026-06-21**

## 🛠 Corrections

- Hotfix CI release : mise à jour du snapshot Ubuntu apt de
  `20260510T000000Z` vers `20260621T000000Z`, afin d'aligner les dépendances
  Linux avec les paquets Mesa présents sur les runners GitHub Actions récents.
- Mise à jour de la version de SONAR en **3.13.22** pour publier ce correctif.

## **[3.13.21] - 2026-06-21**

## 🛠 Corrections

- Hotfix CI release : autorisation explicite des downgrades apt lors de
  l'installation des dépendances Linux depuis le snapshot Ubuntu, afin d'éviter
  les conflits avec les paquets Mesa plus récents déjà présents sur les runners
  GitHub Actions.
- Mise à jour de la version de SONAR en **3.13.21** pour publier ce correctif.

## **[3.13.20] - 2026-06-21**

## 🛠 Corrections

- Correction de la fermeture de l'application depuis la croix de fenêtre, le
  bouton `Quitter`, le raccourci `Ctrl+Q` et le menu natif `Fichier > Fermer`.
- Centralisation de la confirmation de fermeture avec le plugin Tauri `dialog`
  via `ask()`, puis fermeture explicite avec le plugin Tauri `process` via
  `exit(0)`.
- Mise à jour de la version de SONAR en **3.13.20** pour publier ce correctif.

## **[3.13.14] - 2026-06-08**

## 🛠 Corrections

- Normalisation du chemin `rust-src` local vers `/rustc/<commit>` dans
  l'environnement reproductible, afin d'aligner les chemins de la standard
  library Rust entre un poste de dev avec `rust-src` installé et GitHub Actions.
- Mise à jour de la version de SONAR en **3.13.14** pour publier ce correctif.

## **[3.13.13] - 2026-06-08**

## 🛠 Corrections

- Remappage des chemins locaux `rustup` et `cargo` dans l'environnement de build
  reproductible afin de réduire les différences entre un build local et GitHub
  Actions.
- Désactivation explicite des informations de debug et strip des symboles dans
  le profil release Rust.
- Robustesse de la collecte CI du binaire lorsque Cargo laisse l'exécutable dans
  `target/release/deps` après un build Tauri sans bundle.
- Mise à jour de la version de SONAR en **3.13.13** pour publier ce correctif.

## **[3.13.12] - 2026-06-08**

## 🛠 Corrections

- Correction de la vérification de reproductibilité `--no-bundle` : l'absence de
  paquet `.deb` est maintenant traitée comme normale lorsque les bundles ne sont
  pas générés.
- Mise à jour de la version de SONAR en **3.13.12** pour publier ce correctif.

## **[3.13.11] - 2026-06-08**

## 🛠 Corrections

- Activation du fallback apt vers l'archive Ubuntu même lorsque le script est
  exécuté via `sudo`, qui ne préserve pas les variables d'environnement GitHub.
- Mise à jour de la version de SONAR en **3.13.11** pour publier ce correctif.

## **[3.13.10] - 2026-06-08**

## 🛠 Corrections

- Ajout d'un fallback CI vers l'archive Ubuntu standard lorsque
  `snapshot.ubuntu.com` est indisponible, tout en conservant les versions de
  paquets apt pinées.
- Mise à jour de la version de SONAR en **3.13.10** pour publier ce correctif.

## **[3.13.9] - 2026-06-08**

## 🛠 Corrections

- Publication des releases sous forme de binaires reproductibles uniquement,
  sans bundle/installateur.
- Ajout d'une note explicite dans la documentation et le corps de release : sous
  Windows, Npcap doit être installé séparément avant d'utiliser la capture
  réseau.
- Mise à jour de la version de SONAR en **3.13.9** pour publier ce correctif.

## **[3.13.8] - 2026-05-18**

## 🛠 Corrections

- Hotfix release macOS : remplacement de `mapfile` dans le script d'upload des
  bundles Sigstore pour rester compatible avec Bash 3.2 sur les runners macOS.
- Mise à jour de la version de SONAR en **3.13.8** pour publier ce correctif.

## **[3.13.7] - 2026-05-18**

## 🔐 Chaîne de confiance

- Signature des artefacts de release avec `cosign sign-blob` et identité OIDC
  GitHub Actions.
- Publication des bundles Sigstore `.sigstore.json` dans la release GitHub et
  comme artefacts CI.

## 🛠 Corrections

- Remplacement de `subject-checksums` par `subject-path` pour la provenance
  GitHub Artifact Attestations afin d'éviter le parsing incorrect des manifests
  multi-lignes sur Windows.
- Mise à jour de la version de SONAR en **3.13.7** pour publier ce correctif.

## **[3.13.6] - 2026-05-18**

## 🛠 Corrections

- Hotfix provenance Windows : normalisation du fichier
  `release-attestation-subjects-*.sha256` au format `shasum` attendu par
  `actions/attest`, sans marqueur binaire `*`.
- Ajout d'une validation stricte des digests SHA256 avant écriture du manifest
  d'attestation.
- Mise à jour de la version de SONAR en **3.13.6** pour publier ce correctif.

## **[3.13.5] - 2026-05-18**

## ✨ Améliorations

- Ajout des versions de build canonique dans le menu `A propos` : Rust, Node.js,
  Deno et Tauri CLI.
- Injection des versions de build depuis `config/build-versions.env` au moment
  de la compilation Rust.

## 🔐 Chaîne de confiance

- Ajout de la génération d'attestations de provenance GitHub Artifact
  Attestations pour les artefacts de release.
- Génération d'un manifest SHA256 dédié aux sujets d'attestation : binaire,
  bundle plateforme et manifest de hashes de release.
- Mise à jour de la documentation sprint/backlog sur la provenance et la
  séparation entre reproductibilité, signature, SBOM et attestations.
- Mise à jour de la version de SONAR en **3.13.5** pour publier ces changements.

## **[3.13.4] - 2026-05-07**

## 🛠 Corrections

- Hotfix CI release : publication des hashes SHA256 des binaires et des bundles
  directement dans le message de la release GitHub.
- Suppression de l'upload des fichiers `SHA256SUMS-*` comme assets de release.
- Extraction de la génération des hashes et de la mise à jour du message de
  release dans des scripts CI dédiés.
- Mise à jour de la version de SONAR en **3.13.4** pour publier ce correctif.

## **[3.13.3] - 2026-05-07**

## 🛠 Corrections

- Hotfix CI release : génération des fichiers `SHA256SUMS-*` depuis tous les
  répertoires `bundle` produits sous `src-tauri/target`, y compris les builds
  macOS avec target explicite.
- Compatibilité macOS pour le calcul SHA256 via `shasum -a 256` lorsque
  `sha256sum` n'est pas disponible.
- Mise à jour de la version de SONAR en **3.13.3** pour publier ce correctif.

## **[3.13.2] - 2026-05-07**

## 🛠 Corrections

- Hotfix CI release : le contrôle reproductible reste bloquant pour le binaire
  SONAR, mais un bundle `.deb` non reproductible est traité comme diagnostic
  afin de ne pas empêcher la publication d'une nouvelle version.
- Mise à jour de la version de SONAR en **3.13.2** pour publier ce correctif.

## **[3.13.1] - 2026-05-07**

## 🛠 Corrections

- Hotfix vendoring : inclusion complète des fichiers `diagram.svg` et
  `diagram.png` de `libdbus-sys` pour éviter les erreurs de checksum Cargo en
  CI.
- Mise à jour de la version de SONAR en **3.13.1** pour publier ce correctif.

## **[3.13.0] - 2026-05-07**

## ✨ Améliorations

- Mise à jour de la version de SONAR en **3.13.0**.
- Alignement des versions dans `package.json`, `tauri.conf.json`, `Cargo.toml`
  et `Cargo.lock`.

## 🔎 Analyse protocolaire

- Mise à jour de `packet_parser` en **1.3.0** pour ajouter la détection du
  protocole **OPC UA**.

## **[3.12.3] – 2026-04-28**

## 🧪 Build / Reproductibilité

- Verrouillage explicite de la toolchain Rust sur **1.95.0**.
- Ajout de `rust-version = "1.95.0"` dans le manifest Cargo.
- Verrouillage de **Deno 2.7.13** dans le pipeline de release et dans le build
  Docker.
- Déclaration de la version cible de **Node 24.14.0** dans `package.json`.
- Remplacement du bootstrap flottant Node/Deno dans Docker par des versions
  explicitement fixées.
- Ajout d’un `.dockerignore` pour stabiliser et réduire fortement le contexte de
  build Docker.
- Stabilisation partielle des runners GitHub Actions avec des labels plus
  explicites pour macOS et Windows.

## 🛠 Corrections

- Correction de la condition du target macOS dans le workflow de publication
  après changement du runner.
- Mise à jour du backlog et de la revue de sprint liés à l’objectif de build
  reproductible.

## **[3.10.0] – 2026-01-08**

## ✨ Fonctionnalités

- Ajout du parsing Modbus/TCP au niveau applicatif, permettant l’analyse et la
  restitution des communications industrielles OT.

- Introduction d’un mode headless / CLI, permettant l’exécution de SONAR sans
  interface graphique (usage automatisé, serveurs, environnements contraints).

## ✨ Améliorations

- Mise à jour et amélioration de l’installer Npcap pour Windows.

- Mise à jour du release log et du changelog.

## **[3.9.6] – 2025-12-22**

## 🧪 Tests / Packaging

- Tests et ajustements successifs des icônes de l’installer NSIS.
- Corrections de format d’images (PNG → BMP) pour compatibilité NSIS.
- Ajustements visuels et techniques des ressources d’installation.

---

## **[3.9.5] – 2025-12-xx**

## ✨ Améliorations

- Migration complète de l’installer Windows vers **NSIS**.
- Ajout de **Npcap** dans les ressources de l’installer.
- Support de l’installation de Npcap directement depuis l’installer.
- Ajout du support de la langue **française** dans NSIS.
- Nettoyage et stabilisation du pipeline d’installation Windows.

## 🛠 Corrections

- Correction de la détection de Npcap dans l’installer.
- Corrections multiples sur les images utilisées par NSIS (format,
  compatibilité).
- Corrections mineures sur le bundling et les scripts d’installation.

---

## **[3.9.4] – 2025-12-xx**

## 🛠 Corrections

- Suppression du mode **offline install** sur Windows.
- Ajustements liés au bundling Windows.

---

## **[3.9.3] – 2025-12-xx**

## ✨ Améliorations

- Mise à jour de la version de l’application.

## **[3.9.2] – 2025-12-03**

## ✨ Améliorations

- Mise à jour de la version de l’application (`update version`).
- Optimisation de la gestion CPU.

## 🛠 Corrections

- Correction du `.gitignore`.
- Ajustements mineurs dans les statistiques de flux (clarification des logs,
  simplification de `update_flow`).

---

## **[3.9.1] – 2025-12-03**

## ✨ Améliorations

- Mise à jour des dépendances.

## 🖼 Interface

- Correction de l'image CPU affichée dans la top bar.

---

## **[3.9.0] – 2025-12-03**

## ✨ Fonctionnalités

- Ajout d’un système de **loading** lors de l’import PCAP.

---

## **[3.8.3] – 2025-12-01**

## ✨ Améliorations

- Ajout du tag pour la version.

---

## **[3.8.2] – 2025-12-01**

## ✨ Fonctionnalités

- **Amélioration majeure de l’import PCAP**.
- **Refonte du graph processing** pour de meilleures performances et stabilité.

---

## **[3.8.1] – 2025-11-24**

## ✨ Améliorations

- Mise à jour de la crate `packet-parser`.

---

## **[3.8.0] – 2025-11-24**

## ✨ Améliorations

- Mise à jour de la version de l'application.
- Amélioration des logs de démarrage.
- Informations système enrichies.

---

## **[3.7.0] – 2025-11-20**

## ✨ Améliorations

- Mise à jour du parser réseau (`packet-parser`).
- Stabilité accrue dans le traitement des protocoles.

---

## **[3.6.0] – 2025-11-18**

## ✨ Fonctionnalités

- Ajout du **sélecteur d'interface réseau personnalisé**.

## 🎨 Interface

- Ajout d’une **légende flottante** sur le graphe réseau.

---

## **[3.5.0] – 2025-11-14**

## 🎨 Interface

- Ajout d’animations pour les boutons de la barre supérieure.

---

## **[3.4.1] – 2025-11-06**

## ✨ Améliorations

- Mise à jour des dépendances.
- Mise à jour de la documentation.

---

## **[3.4.0] – 2025-10-31**

## ✨ Fonctionnalités

- Ajout d’un **système de filtres amélioré** pour la matrice.

## 🛠 Corrections

- Nettoyage de code inutilisé.

---

## **[3.3.1] – 2025-10-30**

## ✨ Améliorations

- Mise à jour des dépendances.
- Ajustements mineurs du rendu.

---

## **[3.3.0] – 2025-10-29**

## ✨ Fonctionnalités

- Ajout de la **gestion des labels** sur les nœuds du graphe.

## 🎨 Interface

- Améliorations visuelles (zoom, level, clarity).

---

## **[3.2.3] – 2025-10-27**

## 🛠 Technique

- Ajustements internes sur le format des données.

---

## **[3.2.2] – 2025-10-21**

## ✨ Améliorations

- Mise à jour de la gestion des timestamps (`timeval`).

---

## **[3.2.1] – 2025-10-21**

## ✨ Améliorations

- Migration vers **Tauri 2.9**.

---

## **[3.2.0] – 2025-10-20**

## ✨ Fonctionnalités

- Ajout de la **fonction de stop forcé** pour la capture réseau.

---

## **[3.1.0] – 2025-10-20**

## ✨ Fonctionnalités

- Améliorations multiples de stabilité et configuration.

---

## **[3.0.1] – 2025-10-14**

## 🛠 Corrections

- Corrections sur le cycle de release.

---

## **[3.0.0] – 2025-10-14**

## ✨ Fonctionnalités

- Refonte du graphe réseau.
- Suppression de l'ancien système de graphe pour un modèle plus robuste.

---

## [2.4.0] - 2025-11-04

## ✨ Améliorations

- **Gestion des erreurs** : Amélioration de la gestion des erreurs dans les
  commandes réseau
- **Performance** : Optimisation de la gestion des verrous dans `net_capture.rs`
- **Sécurité** : Validation des entrées utilisateur pour les filtres réseau
- **Documentation** : Ajout de la documentation Rust pour toutes les fonctions
  publiques

## 🛠 Corrections

- Correction d'un problème potentiel de fuite de mémoire dans la gestion des
  captures
- Amélioration des messages d'erreur pour faciliter le débogage
- Correction de la gestion des filtres réseau

---

## [2.3.2] - 2025-06-25

## [2.3.2] - 2025-06-25

## ✨ Fonctionnalités

- Ajout de l'affichage des ports sur la vue graphique pour une meilleure
  visibilité des connexions réseau
- Amélioration de la visibilité des protocoles les plus hauts dans la hiérarchie
  réseau
- Optimisation des performances de rendu pour les graphes complexes

## 🛠 Corrections

- Correction de l'affichage des légendes dans la vue graphique
- Amélioration de la stabilité lors de la manipulation des nœuds

---

## [2.2.8] - 2025-05-19

## ✨ Fonctionnalités

Ajout du monitoring CPU en temps réel avec affichage dans la status bar.

Ajout de la fonctionnalité d’export des logs applicatifs depuis le backend Rust.

---

## [2.2.4] - 2025-05-12

## Fix

- Disable config during capture.

---

## [2.2.1] - 2025-05-05

## Fix

- Compatibilité mac os

## [2.2.1] - 2025-05-05

### ✨ Fonctionnalités

- Ajout de la fonctionnalité "stop record".
- Ajout de l’icône `stop.svg` dans `src/assets`.
- Compatibilité améliorée entre Windows 11 et Ubuntu pour les timestamps des
  paquets réseau (`tv_sec`, `tv_usec`).
- Ajout d'une gestion conditionnelle multiplateforme avec
  `#[cfg(target_os = "...")]` pour la conversion des timestamps.

### 🛠 Corrections

- Correction d’un bug de compilation sous Windows 11 (mismatch de types `i32` vs
  `i64`).
- Le fichier `.gitignore` n’ignore plus les `.svg` du dossier `src/assets`.

### 🎨 Interface

- Amélioration de la top bar.
- Amélioration de la status bar.

### 🔧 Technique

- Tag `app-v2.2.0` ajouté à `main` après merge.
- Nettoyage de warnings (`unused import: info`) dans le module `commandes`.
- Suppression de la page de nommage de fichier au démarrage de SONAR. La
  discussion est ouverte pour une réintégration éventuelle au moment de la
  sauvegarde.
- Retrait de la fonctionnalité d'automatisation de la sauvegarde : cette
  fonction n'a jamais été utilisée et ne répondait à aucun besoin identifié
  jusqu'à présent.

## [1.15.0] - 2024-11-07

### NEW

- Intégration de la structure `PacketKey` pour distinguer les paquets sans
  considérer leur taille (`packet_size`) dans la clé, permettant une meilleure
  gestion des doublons et l'accumulation des tailles des paquets dans
  `PacketStats`.
- Ajout de la fonctionnalité de conversion de `PacketKey` en `PacketInfos` pour
  assurer la compatibilité avec les méthodes existantes nécessitant
  `PacketInfos`.

### FIX

- Résolution d'un problème de type qui empêchait l'exportation correcte des
  données de la matrice de paquets vers les fichiers CSV et Excel. Les méthodes
  d'enregistrement ont été adaptées pour utiliser `PacketKey` et `PacketStats`.
- Mise à jour des méthodes du front-end pour traiter correctement la structure
  de l'API, en tenant compte des nouvelles propriétés `infos` et `stats`. Cela
  garantit un affichage précis des données, y compris la taille totale des
  paquets et le nombre d'occurrences.

### IMPROVEMENT

- Refactoring de la méthode `get_matrice_data` pour une sérialisation plus
  claire et un traitement efficace des données.
- Amélioration des journaux de debug pour une meilleure traçabilité des paquets
  et de leur traitement dans l'application.

Cette version améliore la gestion des paquets avec des tailles différentes, la
stabilité et la clarté du code tout en offrant une meilleure expérience
utilisateur dans l'interface de visualisation.

---

## [1.14.1] - 2024-10-31

### FIX

Reload ... ! Résolution d'une erreur de parsing des paquets DNS qui provoquait
le blocage de l'application Sonar. Ce correctif améliore la stabilité et la
fiabilité de l'analyse des paquets DNS.

---

## [1.12.0] - 2024-07-04

### Ajouté

parse pacquet 7

---

## [1.11.1] - 2024-05-17

### Ajouté

Pipeline cicd pour raspberry pi

---

## [1.11.0] - 2024-05-02

### Ajouté

- Affichage des adresses IP publiques dans la vue graphique. Cela permet aux
  utilisateurs de visualiser les adresses IP directement depuis l'interface
  graphique de l'application.

### Modifié

- Modification de l'entrée pour la durée du relevé dans l'interface utilisateur
  de Vue.js pour accepter des valeurs jusqu'à 48 heures. Auparavant, l'entrée
  était limitée à 24 heures.
- Adaptation du type d'entrée pour la durée de relevé de `type="time"` à
  `type="text"` pour permettre la saisie manuelle de la durée en format
  "HH:MM:SS", permettant ainsi de saisir des durées supérieures à 24 heures.
- Mise à jour de la fonction `validateTime` pour valider les heures, les minutes
  et les secondes manuellement en utilisant une nouvelle logique qui supporte
  jusqu'à 48 heures.

### Corrigé

- Mise à jour de la fonction de récupération des informations système pour
  utiliser `whoami` via Rust et traiter la sortie pour obtenir spécifiquement le
  nom de la machine et la version du noyau.

---

## [1.9.0] - 2024-03-20

### Nouvelles fonctionnalités

- **Tableau avec vutify**:

---

## [1.8.0] - 2024-03-19

### Nouvelles fonctionnalités

- **Visualisation des Réseaux**: Implémentation d'une fonctionnalité de
  visualisation de réseaux améliorée, offrant des vues en courbes pour les
  connexions et un système de couleurs dynamique basé sur les types de
  protocoles.

---

## [1.7.0] - 2024-03-18

### Nouvelles fonctionnalités

- **Type d'IP** : Implémentation d'une nouvelle fonctionnalité permettant de
  déterminer le type d'une adresse IP (privée, APIPA, multicast, loopback,
  lien-local, ULA, publique ou inconnue) à partir d'une chaîne de caractères.
  Cette amélioration apporte une capacité critique à l'analyse et à la
  classification des adresses IP dans divers contextes de réseau.

### Améliorations

- **Détection des adresses APIPA** : Amélioration de la précision dans la
  détection des adresses IP APIPA (Automatic Private IP Addressing), permettant
  une identification plus fiable des appareils configurés automatiquement sans
  serveur DHCP.

- **Support Multicast IPv4** : Extension du support pour identifier les adresses
  multicast IPv4, facilitant la gestion et le filtrage des paquets destinés à
  des groupes d'écoute multicast.

- **Prise en charge IPv6** : Renforcement de la prise en charge des adresses
  IPv6 avec l'identification spécifique des adresses lien-local et ULA (Unique
  Local Address), améliorant ainsi la capacité à traiter et analyser le trafic
  IPv6 moderne.

### Corrections de bugs

- **Correction de la classification Loopback IPv6** : Résolution d'un problème
  où les adresses loopback IPv6 (`::1`) étaient incorrectement classifiées comme
  publiques, assurant désormais une identification correcte comme adresses
  loopback.

### Documentation

- **Mise à jour de la documentation** : Ajout de documentation pour la nouvelle
  fonctionnalité de type d'IP, incluant des exemples d'utilisation et des
  descriptions des différents types d'adresses IP supportés.

### Tests

- **Amélioration des tests unitaires** : Ajout et mise à jour de tests unitaires
  pour couvrir les nouvelles fonctionnalités et améliorations, notamment pour la
  détection des types d'adresses IP et la correction de la classification des
  adresses IPv6 loopback.

---
## [1.6.0] - 2024-02-26

### UI/UX

- Tableau des trames en temps réel présentant désormais 5 lignes vides par défaut pour une meilleure visibilité initiale.
- Ajustement de la hauteur des lignes du tableau des trames en temps réel pour améliorer la cohérence visuelle.

### Nouvelles fonctionnalités

- **Filtre ip** : Ajout d'un filtre pour IPv4 permettant une meilleure catégorisation et recherche des trames réseau.
- **rm lo on linux** :
---

## [1.5.0] - 2024-02-15

### Nouvelles fonctionnalités

- **colonne l7** :
- **documentation**

---

## [1.4.0] - 2024-02-15

### Corrections de bugs

---

## [1.3.3] - 2024-02-15

### Corrections de bugs

- **Liste des interfaces sur Windows** : Correction d'un problème où les noms
  des interfaces réseau étaient mal affichés sur Windows, apparaissant comme des
  UUID au lieu de noms conviviaux. Maintenant, les adresses MAC des interfaces
  sont utilisées pour permettre une identification plus aisée des interfaces
  réseau sur cette plateforme.

---

## [1.3.2] - 2024-02-13

### Nouvelles fonctionnalités

- **Ajout de code coverage** : Implémentation d'outils de couverture de code
  pour garantir la qualité des suites de tests et identifier les parties du code
  non testées.

---

## [1.3.1] - 2024-02-13

### Nouvelles fonctionnalités

- **Ajout de la colonne Packet Size** : Une nouvelle colonne pour la taille des
  paquets a été ajoutée pour fournir plus de détails sur chaque paquet capturé.
  Cela permet une analyse plus approfondie du trafic réseau en offrant une
  visibilité sur la taille des paquets en plus de leurs métadonnées existantes.

---

#### Version 1.2.1

**Nouvelles fonctionnalités :**

- **info bulle avec ip sur les nodes**

---

#### Version 1.1.1

**Nouvelles fonctionnalités :**

- **Enregistrement de la vue graphique au format SVG :** Il est désormais
  possible d'enregistrer la vue graphique de vos données réseau au format SVG.
  Cette fonctionnalité permet une préservation de haute qualité de vos
  visualisations pour une utilisation dans des rapports ou des présentations.
  Pour sauvegarder votre visualisation, sélectionnez l'option 'Sauvegarder en
  SVG' depuis la vue graphique.

- **Affichage des protocoles sur les arêtes :** Les visualisations graphiques
  ont été améliorées pour afficher les protocoles qui interagissent entre les
  adresses MAC. Cette mise à jour enrichit l'analyse en offrant une
  compréhension immédiate des types de communications se déroulant au sein de
  votre réseau, permettant ainsi d'identifier plus facilement les modèles de
  trafic et les éventuelles anomalies.

---

#### Version 1.1.0

**Nouvelles fonctionnalités :**

- **Sauvegarde au format Excel :** Vous pouvez maintenant sauvegarder vos
  données non seulement au format CSV, mais également au format Excel (XLSX).
  Cette option offre une plus grande flexibilité pour le traitement et l'analyse
  des données en dehors de l'application. Pour utiliser cette fonctionnalité,
  sélectionnez simplement l'option 'Sauvegarder en Excel' dans la section de
  sauvegarde des données.
- **Vue Graphique :** Une nouvelle fonctionnalité de visualisation graphique a
  été ajoutée pour vous permettre de voir les tendances et les analyses de vos
  données de manière plus intuitive. Accédez à des graphiques dynamiques et
  interactifs qui présentent vos données de réseau de manière visuelle,
  facilitant ainsi la compréhension et l'interprétation des informations
  complexes.

---

#### Version 1.0.1

**Nouvelles fonctionnalités :**

- **Gestion TCP/IP :** Sonar inclut désormais des capacités améliorées pour la
  gestion des protocoles TCP/IP. Cette fonctionnalité vise à améliorer l'aspect
  communication réseau du logiciel, en assurant un transfert de données plus
  robuste et efficace sur le réseau.

- **Sauvegarde en CSV :** Une nouvelle fonctionnalité a été ajoutée pour
  permettre aux utilisateurs d'exporter des données au format CSV
  (Comma-Separated Values). Cette fonctionnalité est particulièrement utile pour
  l'analyse de données et la création de rapports, car elle permet une
  manipulation facile des données et une intégration avec divers outils qui
  prennent en charge le CSV.

**Améliorations :**

- Optimisations générales des performances de l'application principale.
- Amélioration de l'interface utilisateur pour une meilleure facilité
  d'utilisation.

**Corrections de bugs :**

- Correction de bugs mineurs concernant des problèmes signalés dans la version
  précédente.

---

#### Version 1.0.0

**Première publication :**

- Implémentation des fonctionnalités de base de Sonar.
- Les fonctionnalités principales incluent des pratiques de développement Agile,
  une intégration avec GitHub pour le contrôle de version, et un accent sur Rust
  pour la performance et la fiabilité.
- Mise en place initiale des protocoles de test et d'assurance qualité.
- Mise en place de la documentation avec des fichiers markdown pour les README
  et les directives de contribution.
- Stratégie d'intégration front-end et back-end utilisant Tauri et Vue.js.
