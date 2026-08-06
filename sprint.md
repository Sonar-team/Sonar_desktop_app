# Sprint P0 — Sessions persistantes : projets, autosave et récupération

> Statut : actif (démarré le 03/08/2026) — dernière revue : 06/08/2026
> Suivi GitHub : [#159](https://github.com/Sonar-team/Sonar_desktop_app/issues/159)
> Sprint précédent (fidélité des données, #165) : essentiel livré, reliquat
> suivi dans `todo.md` — #154 (identité d'actif, prérequis de la phase B
> ci-dessous), corpus/fuzzing #151, validations #88. Historique détaillé
> dans l'issue #165 et le sprint.md antérieur (git, commit `174cb96b`).

## Objectif

Aucun travail utilisateur perdu sur stop, fermeture ou crash : SONAR
fonctionne par projets, comme un outil de bureau professionnel, et sait
prouver l'origine des données analysées (#159).

## Conception

Format de projet `.sonar` = archive ZIP à entrées fixes :
`manifest.json` (schema_version, versions, DLT, comptages),
`matrice.csv` (format SFMS, writer déterministe existant),
`labels.csv` (writer existant), `capture.json` (config + filtre BPF).
L'ouverture compose les chemins d'import existants (swap transactionnel
matrice+graphe de `import_matrix_files`, remplacement du store par
`import_label_file`) — aucune logique d'état réécrite. Spécification
complète en commentaire de #159.

## Phase A — projet manuel et fondations *(sans dépendance à #154)*

1. [x] Format `.sonar` v1 + commandes `save_project`/`open_project` :
   écriture atomique (staging + `.part` + rename), manifest validé avant
   toute mutation, schéma futur refusé avec message d'action, config
   revalidée par bornes. Boutons TopBar 🗃️/📂. *(03/08, PR du sprint)*
2. [x] `capture_config.json` écrit atomiquement (dernier fichier qui ne
   l'était pas). *(03/08, même PR)*
3. [x] État dirty : révision dans `CaptureState` (marquée par démarrage de
   capture, imports, éditions de labels ; blanchie par reset, save, open —
   les modifications pendant une écriture longue restent comptées),
   commande `is_session_dirty`, confirmation avant reset et fermeture
   seulement si modifié. *(03/08, même PR)*
4. [~] Arrêt gracieux à la fermeture : `stop_capture` (drainage #158 +
   jointure) avant `exit`, best-effort. *(03/08 ; la « sauvegarde finale »
   arrive avec l'autosave, point 5)*
5. [x] Autosave : thread périodique (60 s) piloté par la révision — n'écrit
   que si le relevé a changé, jamais pendant un import, snapshot sous
   verrou court, écriture atomique, ne blanchit pas le suivi dirty.
   *(04/08 ; déclenchement sur événements clés et intervalle adaptatif :
   améliorations possibles, non bloquantes)*
6. [x] Récupération après crash : sentinelle `session.lock` posée au setup,
   retirée sur `RunEvent::Exit` ; sentinelle restante + autosave présent au
   démarrage → dialogue de récupération (App.vue) → `open_project`.
   *(04/08)*
7. [x] Projets récents via `tauri-plugin-store` 2.4.4 (vetté, vendoré,
   `store:default` ajouté à la capability et à son test de sécurité) :
   dialogues save/open préremplis avec le dernier dossier, liste des 10
   derniers projets persistée. Vérification faite : le plugin écrit en
   `fs::write` NON atomique → il ne porte que des préférences UI, la
   config de capture reste sur notre persistance atomique Rust. *(04/08)*

**Phase A terminée le 04/08/2026.**

## Phase B — identité et migration *(après #154)*

> Le format est posé côté crate depuis le 06/08 : `SurveyContext
> { site, sensor, interface }` dans le préambule SFMS et `SFMS_VERSION` 2
> (sonar-flows-core 0.6.0, PR #189). Le contexte reste **hors de la clé**
> des flux et des nœuds : le même paquet peut être vu par plusieurs
> capteurs, et le mettre dans la clé empêcherait de reconnaître ce
> recouvrement. Reste le câblage desktop, ci-dessous.

8. [ ] Saisie du contexte de relevé au stop/save, à la manière de
   Wireshark — pas de configuration a priori (arbitrage du 04/08).
9. [ ] Identité d'actif contextualisée dans le schéma → `schema_version 2`,
   premier test de migration réel v1 → v2.
10. [ ] Généralisation d'`origin` en contexte par flux, pour la fusion
    multi-sites.

## Phase C — manifest de preuve

11. [ ] Hashes SHA-256 des entrées calculés à l'import, compteurs qualité
    (#150), hashes des sorties ; manifest exportable seul et vérifiable
    hors application (script fourni).
12. [ ] Signature du manifest (ed25519/minisign ; gestion de clé à
    trancher — candidat `tauri-plugin-stronghold`, statut amont IOTA à
    vetter avant tout engagement supply-chain).

## Definition of Done (critères de #159)

- [x] Un projet sauvegardé se rouvre avec matrice, graphe, labels et
  métadonnées identiques. *(test bout-en-bout via les vraies commandes
  IPC : import réel → save → état vidé → open → matrice identique)*
- [x] Une écriture interrompue ne corrompt jamais le dernier projet valide.
  *(phase A.1 : `.part` + rename, testé)*
- [x] Un crash simulé propose la récupération du dernier checkpoint.
  *(détection sentinelle+autosave testée unitairement ; à rejouer en E2E
  réel lors de la validation #146)*
- [x] Reset et fermeture demandent confirmation si l'état est modifié.
  *(phase A.3 ; la croix de fenêtre passait déjà par `onCloseRequested`)*
- [x] Le format est versionné… *(v1 + refus des schémas futurs, testé)* —
  [ ] …et possède des tests de migration *(dès la v2, phase B)*.
- [ ] Le bundle de preuve est déterministe et vérifiable hors de
  l'application. *(phase C)*

## Travail parallèle autorisé

- reliquat fidélité : #154 (préalable de la phase B — côté crate livré le
  06/08, reste le câblage desktop), corpus/fuzzing #151 ;
- conception de #160 (matrice de production), sans mordre sur la phase A.

*(#175, Depends libpcap du .deb, était listé ici : fermé le 04/08.)*

## Hors périmètre

- distribution : #94, #136, #146, #162 ;
- différenciation : #164, #156 ;
- le contenu complet du manifest de preuve dépend de la comptabilité #150
  et du drainage sans perte — les compteurs manquants seront ajoutés quand
  #150 sera formellement close.
