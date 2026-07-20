# Sprint P0 — Fidélité des données et intégrité des sessions

> Statut : actif — orange
> Dernière synchronisation : 20/07/2026
> Avancement du plan initial : 6 lots sur 9 ; Definition of Done : 6/10
> Référence vérifiée : `main` au SHA `a8d8b5a6`
> Sources : audits bêta → pro des 13/07 et 17/07/2026
> Suivi GitHub : [#165](https://github.com/Sonar-team/Sonar_desktop_app/issues/165)
> Priorisation :
> [project_management/priorisation_beta_to_pro.md](project_management/priorisation_beta_to_pro.md)

## Objectif

Garantir qu'aucun paquet, flux ou état de session ne puisse être perdu,
ignoré, fusionné ou remplacé silencieusement.

## Phase 0 — rendre les défauts observables

1. [x] [#87](https://github.com/Sonar-team/Sonar_desktop_app/issues/87) :
   reproduire l'import infini ou le fermer avec preuve et test. *(14/07 :
   fermée non reproductible — preuves de terminaison backend testées, cause
   plausible restante côté UI suivie dans #161.)*
2. [x] [#151](https://github.com/Sonar-team/Sonar_desktop_app/issues/151) :
   supprimer tout succès obtenu en sautant une fixture absente. *(14/07 :
   LOC42 remplacé par le corpus public nDPI, plus aucun skip silencieux ;
   reste le fuzzing en phase 1.)*
3. [ ] [#88](https://github.com/Sonar-team/Sonar_desktop_app/issues/88) :
   revalider espaces/Unicode et créer les tests nécessaires. *(14/07 :
   backend testé — PCAP/matrices/labels, espaces/Unicode/`'`/`"`/`` ` `` ;
   restent les tests Windows et front.)*

## Phase 1 — atomicité et comptabilité

4. [x] [#139](https://github.com/Sonar-team/Sonar_desktop_app/issues/139) :
   réserver `Importing` pendant toute la conversion. *(14/07 : phase
   `Importing` + guard RAII, tests de course déterministes. Le refus du reset
   concurrent et des imports vides est désormais suivi explicitement par
   [#167](https://github.com/Sonar-team/Sonar_desktop_app/issues/167)).*
5. [x] [#150](https://github.com/Sonar-team/Sonar_desktop_app/issues/150) :
   définir le résultat canonique de parsing et le rapport qualité.
   *(14-15/07 : DLT réel, refus avant mutation, préambule `#SFMS`, rapport
   qualité visible et DLT documentés. Round-trip d'identité RAW/SLL/SLL2
   fermé sans `link_details` : les métadonnées cooked du point d'observation
   ne segmentent pas les conversations multi-sondes ; fixtures réelles SLL et
   SLL2 testées bout en bout. Ce lot du sprint est livré, mais #150 reste
   ouverte pour les catégories fines, l'export/projet et la preuve exhaustive.)*
6. [x] [#158](https://github.com/Sonar-team/Sonar_desktop_app/issues/158) :
   drainer arrêt et plafond avec des compteurs exacts. *(14/07 : drainage au
   plafond, compteurs intégrés/illisibles, récapitulatif final.)*
7. [ ] [#151](https://github.com/Sonar-team/Sonar_desktop_app/issues/151) :
   compléter multi-DLT, malformé, PCAPNG et fuzzing. La preuve différentielle
   PCAP → matrice est suivie dans
   [#168](https://github.com/Sonar-team/Sonar_desktop_app/issues/168).

## Phase 2 — intégration et identité

8. [x] [#142](https://github.com/Sonar-team/Sonar_desktop_app/issues/142) :
   générer et tester le contrat IPC Rust → TypeScript. *(15/07, revue puis
   corrigée le 15/07 après un audit ayant trouvé 6 défauts de fidélité dans
   la première passe — voir le commentaire de réouverture sur l'issue.
   État final : conversion `to_contract` exhaustive (`match` sans `_` entre
   `CaptureEvent` réel et son miroir — un compilateur, pas un rappel humain,
   impose sa mise à jour), `#[ts(optional)]`/`#[ts(optional = nullable)]`
   sur tous les champs réellement omissibles (un paquet sans VLAN/couche
   applicative satisfait maintenant son propre type TS), variante `Packet`
   morte supprimée des deux côtés plutôt que mal mirée, gate CI réécrite
   (`git status --porcelain` après suppression + régénération complète, pas
   `git diff` seul — qui laissait passer silencieusement les 18 nouveaux
   fichiers non suivis), `protocol_version` vérifié par le store et émis par
   les trois chemins de session (capture live, import PCAP, import matrice
   CSV — ce dernier n'émettait pas `Started` avant), tag inconnu journalisé
   au lieu de faire planter le store. Bug annexe corrigé : `BottomLong.vue`
   s'abonnait à `onPacket` ET `onPacketBatch`, doublant chaque trame
   affichée. 18 tests Rust de fidélité JSON (dont paquet sans VLAN,
   tunnelé, couche corrompue, groupes internet/transport/application tous
   absents). *(Deuxième revue, 15/07 : 6 défauts supplémentaires trouvés et
   corrigés — `encap_id` paquet en hex au lieu de `number` (hash 64 bits,
   perte de précision possible au-delà de `Number.MAX_SAFE_INTEGER`),
   `Started`/version ajoutés à l'import de labels, `BottomLong.vue`
   discriminé sur `link_kind` au lieu d'un cast vers un type `unknown`
   local, `any` éliminé de `normalizeGraphUpdate`/`nodeAttributes`/
   `edgeAttributes`/reducers Sigma/`applyUpdate` (switch `never` ajouté),
   `src/types/captureContractFixtures.ts` fait vérifier le JSON par
   TypeScript (`deno task typecheck`, CI) — le vrai test Rust → JSON →
   TypeScript qui manquait, 5 tests de fidélité JSON ajoutés côté erreurs.
   Découverte annexe majeure : `deno task typecheck` ne détectait plus
   aucune erreur depuis un temps indéterminé — `deno_task_shell` avale la
   sortie/le code de sortie de `vue-tsc` résolu par son nom nu ; corrigé en
   invoquant `node node_modules/.bin/vue-tsc` (`deno.json`). Le job CI
   « Gates frontend » ne contrôlait donc jamais réellement le frontend.)*
   *(Troisième revue, 15/07 : la preuve Rust → JSON → TypeScript reposait
   sur des littéraux retapés à la main — `encap_id` toujours `null` dans les
   deux fixtures, une régression `string`→`number` restait invisible.
   `cargo test export_ipc_fixtures` écrit désormais le JSON réellement
   produit par `CaptureEvent` (17 fixtures, dont `encap_id` non nul, SLL2 et
   IEEE 802.11 en `packetBatch` complet, `NodeUpdated`/`EdgeUpdated`) ;
   régression simulée et confirmée détectée. `any` éliminé de
   `normalizeGraphUpdate` (→ `unknown` + narrowing), `nodeAttributes`/
   `edgeAttributes`/`refreshParallelEdges`/`drawNodeLabel` (types Sigma/
   graphology dédiés) ; `StatusBar.vue` passé en `lang="ts"` (bug mort
   trouvé : `this.matrice_len` n'existait nulle part, supprimé) ; import de
   labels vérifie `protocol_version` aussi. Le typecheck strict sur
   `src/tests/` (Deno) est activé depuis la revue suivante (`--no-check`
   retiré) : la tension d'architecture supposée ci-dessus n'en était pas
   une — corrigé uniquement les fixtures de `graphSync.test.ts` devenues
   invalides à cause du typage plus strict introduit ici.)*
   *(Quatrième revue, 15/07 : gate CI corrigée pour lancer aussi
   `export_ipc_fixtures` (elle ne lançait que `export_ipc_bindings`, laissant
   `captureEventFixtures.ts` perpétuellement absent après le `rm -rf` de la
   gate) ; IDs de `Node`/`Edge` de test fixés (dépendaient d'un compteur
   atomique global partagé par tout le binaire de test, donc de l'ordre
   d'exécution des autres tests) — génération vérifiée déterministe sur
   plusieurs exécutions, y compris après la suite complète ; dossier des
   fixtures créé explicitement (`create_dir_all`, condition de course avec
   `export_ipc_bindings` sinon) ; `--no-check` retiré de `deno task test` et
   les 7 erreurs de typage strict qui en découlaient corrigées ; fixtures
   étendues aux 9 variantes d'`IpType`, aux `NetworkProtocol` `Ipv6`/`Arp`/
   `Profinet`/`Other` et aux deux variantes de `CorruptedLayerKind` (seuls
   `Private`/`Public`/`Ipv4` étaient couverts côté TypeScript) ;
   `captureStore.test.ts` typait ses événements simulés en `unknown` — typé
   en `CaptureEvent`, ce qui a révélé 11 événements de test incomplets/d'un
   format antérieur, remplacés par du JSON réellement dérivé des fixtures
   Rust ; `normalizeGraphUpdate` (5 casts `as unknown as` sur un format
   `NewNode`/`NewEdge` jamais produit par le contrat actuel — le seul
   appelant recevait déjà un `GraphUpdate` typé) supprimé, `ChannelStatus.vue`
   dérive maintenant son payload du contrat généré au lieu d'une interface
   locale incomplète (`session_id`/`backpressure` manquants, invisible côté
   TypeScript à cause de la vérification bivariante des méthodes) ; `Stats.
   integrated`, `Finished.file_name`/`integrated_count` et `Stopped.reason`
   affichés dans `StatusBar.vue` (auparavant ignorés ou seulement
   journalisés).)*
   *(Cinquième revue, 15/07 : convention de sérialisation unifiée en
   `camelCase` (tags ET champs, `rename_all`/`rename_all_fields`) sur tout ce
   qui est Sonar-owned — `CaptureEvent` (`events/mod.rs`), `StatsPayload`,
   `CapturedPacketOwned` (`ts_sec`/`ts_usec`/`encap_id` →
   `tsSec`/`tsUsec`/`encapId`), `Node`/`Edge`/`GraphUpdate` de
   `sonar_flows_core` (`source_port`→`sourcePort`, tags `NodeAdded`→
   `nodeAdded`…) et leurs miroirs dans `contract.rs`. Seule exception
   assumée et documentée en tête de `events/mod.rs` : la couche paquet/flux
   qui traverse la crate vendorée `packet_parser` (`PacketFlow`, `DataLink`,
   `IpType`, `NetworkProtocol`, `CorruptedLayerKind`) reste dans la casse
   qu'elle impose (`snake_case` pour les champs, mélange snake_case/
   PascalCase pour les tags selon l'enum) — cette crate ne se modifie jamais
   ici (cf. `never-edit-vendor-packet-parser`) ; à signaler en amont si une
   uniformisation complète devient un jour nécessaire. Les 109 tests
   src-tauri, 63 tests sonar-flows-core et 38 tests frontend stricts restent
   verts après le renommage ; aucune dérive entre le JSON réellement produit
   et les mirrors `contract.rs` (vérifié par `cargo test`).)*
   *(Sixième revue, 15/07 : fixtures TS étendues aux nullabilités jamais
   exercées — nœud sans label, arête sans port service
   (`graphBatchWithoutLabelOrPortsFixture`), groupe internet/transport
   *présent* mais champs individuels absents plutôt que le groupe entier
   (`packetBatchInternetAndTransportPresentWithoutAddressesFixture`, protocoles
   `ARP`/`ICMP`), SLL sans adresse source
   (`packetBatchSllWithoutAddressFixture`) — avec leur pendant côté tests de
   fidélité JSON Rust. Mutation simulée et confirmée détectée : retirer
   `| null` de `Node.label`, `Edge.sourcePort`/`destinationPort` ou
   `PacketFlow.source_ip`/`source_port` fait échouer `deno task typecheck`
   sur ces fixtures.)*
   *(Septième revue, 15/07 : faille de preuve trouvée et corrigée — le
   helper générique `fx<T extends CaptureEvent>(v: T): T` (inférence de `T`
   depuis l'argument lui-même) ne détectait pas les champs excédentaires.
   Contre-test confirmé avant fix : `#[ts(skip)]` sur `backpressure` retire
   le champ du binding TS généré tout en le laissant dans le JSON réel (le
   `#[ts(skip)]` n'affecte pas `serde`) — les 111 tests Rust, `deno task
   typecheck` et les 38 tests frontend restaient verts. Remplacé par
   `satisfies CaptureEvent` (TypeScript 4.9+), qui vérifie les champs
   manquants ET excédentaires à l'assignation tout en conservant le type
   littéral précis de chaque fixture pour ses consommateurs ; contre-test
   rejoué après fix : `deno task typecheck` échoue bien sur le même
   `#[ts(skip)]`. SLL2 sans adresse source ajouté
   (`packetBatchSll2WithoutAddressFixture`, même trou que SLL déjà couvert
   à la revue précédente), avec son pendant Rust. Documentation obsolète
   corrigée : mention `as const` dans `captureContractFixtures.ts` (le
   fichier utilise `satisfies` depuis cette revue), mention d'un événement
   `packet` mono-paquet supprimé dans `capture.ts` (n'existe plus sur le
   fil). `ChannelStatus.vue` utilise maintenant `backpressure` (barre rouge
   + titre quand actif) au lieu de ne recevoir que le reste du payload.
   Convention toujours hybride au sens strict de l'énoncé #142 (exception
   `packet_parser` déjà actée et documentée à la cinquième revue, pas
   revisitée ici faute de nouvelle décision). Livré : commit `45778e61`
   poussé sur `origin/main`, CI distante GitHub Actions verte (Rust CI —
   dont la gate « Check IPC contract is up to date » —, coverage, trivy,
   rust-clippy analyze, CodeQL, SonarCube).)*
   *(Huitième revue, 15/07 : garde « champ fantôme » ajoutée — un champ
   optionnel ajouté à la main dans un type de contrat (`CaptureEventContract`,
   `NodeContract`…) sans provenir d'aucune donnée réelle reste toujours
   `None`, donc toujours omis des deux côtés, réel et miroir ; aucun test de
   fidélité JSON existant ne pouvait le remarquer. Contre-test confirmé
   avant fix : champ `ghost: Option<bool>` ajouté à `Started`, toujours
   `None` dans `to_contract` — les 22 tests de contrat restaient verts.
   Nouveau test `assert_no_phantom_contract_fields` (dans
   `export_ipc_fixtures`) : extrait les champs optionnels de chaque
   déclaration `.decl()` ts-rs, les compare à l'ensemble des champs
   réellement présents (non `null`) dans tous les fixtures combinés, échoue
   si un champ optionnel déclaré n'apparaît jamais. Contre-test rejoué :
   échoue bien sur `ghost`. Heuristique de couverture, pas une garantie
   structurelle (choix assumé — l'alternative, dériver `ts_rs::TS`
   directement sur les types réels `CaptureEvent`/`Node`/`Edge`/
   `GraphUpdate` au lieu de miroirs recopiés, a été écartée comme trop
   large pour cette revue).)*
9. [ ] [#154](https://github.com/Sonar-team/Sonar_desktop_app/issues/154) :
   stabiliser l'identité d'actif contextualisée.

## Risques P0 apparus pendant le sprint

Ces défauts ont été confirmés sur le code courant pendant la synchronisation
du 20/07. Ils s'ajoutent aux garanties à fermer sans modifier le décompte des
neuf lots du plan initial.

- [ ] [#166](https://github.com/Sonar-team/Sonar_desktop_app/issues/166) :
  supprimer l'interblocage possible entre démarrage et arrêt concurrents ;
- [ ] [#167](https://github.com/Sonar-team/Sonar_desktop_app/issues/167) :
  refuser les imports vides et les resets concurrents sans perdre la session.

## Chantier qualité rattaché

- [ ] [#168](https://github.com/Sonar-team/Sonar_desktop_app/issues/168) — P1 :
  prouver l'exactitude PCAP → matrice avec une vérité terrain TShark hors ligne.

## Livrables

- classification canonique partagée cœur, CLI et desktop ;
- égalité vérifiable entre paquets lus et toutes les catégories ;
- pertes noyau, interface et application distinguées ;
- import protégé contre une capture ou un second import concurrent ;
- reset concurrent et import vide refusés avant mutation (#167) ;
- arrêt sans paquet accepté abandonné silencieusement ;
- corpus assaini ou généré déterministement ;
- contrat IPC généré et exhaustif ;
- identité tenant compte du projet/site, capteur, interface et VLAN.

## Travail parallèle autorisé

- [#161](https://github.com/Sonar-team/Sonar_desktop_app/issues/161) :
  double batch, déduplication des fichiers et déverrouillage `finally` ;
- [#162](https://github.com/Sonar-team/Sonar_desktop_app/issues/162) :
  workflow qualité commun aux PR et releases ;
- [#168](https://github.com/Sonar-team/Sonar_desktop_app/issues/168) :
  préparation des fixtures et de la vérité terrain TShark ;
- conception de [#159](https://github.com/Sonar-team/Sonar_desktop_app/issues/159),
  sans figer son schéma avant #154.

## Definition of Done

- [ ] Chaque paquet lu appartient à une catégorie explicite.
- [x] Un DLT non supporté échoue avant toute mutation de l'état. *(v4.5.0)*
- [x] Une capture ne peut pas démarrer pendant un import. *(#139, 14/07)*
- [x] Stop et limite de flux drainent ou comptent la perte exacte. *(#158, 14/07)*
- [x] Aucun test critique ne dépend silencieusement d'un fichier local. *(#151, 14/07)*
- [x] Le rapport final traverse un IPC généré et est visible. *(#142, 15/07 ;
  export dédié du rapport non couvert — l'export CSV de la matrice existe déjà)*
- [ ] Deux actifs de même IP sur des VLAN/sites distincts ne sont pas fusionnés.
- [ ] Les courses et chemins d'arrêt ont des tests déterministes complets
  (#166 et #167 restent ouverts).
- [ ] Typecheck, tests, builds, fmt et Clippy strict sont verts sur le SHA
  courant.
- [x] Les DLT supportés et limites sont documentés. *(README, 15/07)*

## Hors périmètre

- produit et persistance : #159, #160, #161 ;
- distribution : #94, #138, #146, #162 ;
- documentation/support : #163 ;
- différenciation : #164.
