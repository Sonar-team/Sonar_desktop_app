# Analyse complète du frontend — 01/08/2026 (Claude Sonnet 5)

Périmètre : `src/` (Vue 3 Composition/Options API + TypeScript + Pinia + Sigma.js/Graphology), hors `src/types/generated/*` (généré depuis Rust via ts-rs, non audité ligne à ligne). ~7 800 lignes hors fichiers générés/tests. Tous les fichiers listés ont été lus intégralement (pas de grep superficiel) ; les findings à sévérité haute ont été re-vérifiés manuellement (lecture directe + pour deux d'entre eux, exécution réelle via `deno test`).

Repo propre sur `main`, aucun fichier modifié pendant l'analyse.

## Méthode

Revue en 5 lots parallèles (panneaux AnalyseView, graphe Sigma/Graphology, NavBar/status-bar, cœur store/erreurs/types/utils/router, tests+tooling), chacun avec instruction de ne remonter que des défauts vérifiés en lisant le code réel — plusieurs points d'une analyse précédente (23/07) ont été explicitement retestés et se sont révélés corrigés depuis (routes mortes, `Matrice.vue`, ancien test Jest — tous supprimés par le commit `415298c7`).

---

## Findings — sévérité haute

### 1. `src/errors/capture.ts:70-71` — crash si l'erreur `capture` imbriquée n'est pas un objet
```ts
const captureKind = captureError.message as CaptureErrorKind;
if ("kind" in captureKind) {   // lève TypeError si captureKind est null/string/undefined
```
Vérifié par exécution : `displayCaptureError({ kind: "capture", message: null })` lève `TypeError: Cannot use 'in' operator...`. La garde ajoutée pour #161 (ligne 44) protège l'enveloppe externe mais pas ce payload imbriqué — contrairement à `handleExportError`/`handleImportError`/`handleLabelerror` qui ont bien `typeof !== "object"`. `displayCaptureError` est `async` et beaucoup d'appelants ne l'attendent pas (`TopBar.vue:193,275,297`, `LabelsPanel.vue:169,189,...`) : l'exception devient une rejection non gérée, **aucun dialogue d'erreur ne s'affiche** à l'utilisateur en cas de dérive de contrat côté backend.
**Correctif** : appliquer la même garde `typeof/null` à `captureKind` que celle utilisée dans `isImportCancellation` (ligne 197-203), qui est le bon modèle.

### 2. `src/utils/labelImport.ts:22` — même défaut pour les erreurs de label
```ts
const labelError = error.message as LabelErrorKind;
switch (labelError.kind) {   // lève si error.message est null
```
Vérifié par exécution : `classifyLabelImportError({ kind: "label", message: null })` lève `TypeError: Cannot read properties of null`. Appelé en synchrone depuis `ImportPanel.vue:453-475` (`catch (err) { this.showLabelFileIssues(err); }`) : si ça lève, le `finally` libère l'UI mais ni le dialogue de conflits ni un message de repli ne s'affichent — le panneau se libère silencieusement sans expliquer l'échec.

### 3. `src/components/AnalyseView/NetworkGraphComponent.vue:333-338` — `resetGraph()` ne vide pas la queue d'updates
```ts
resetGraph() {
  this.forceLayout?.kill()
  this.graph?.clear()
  this.clearNodeInfos()
  this.unpinTunnelHighlight()
}
```
`_queue` (alimentée par `onGraphUpdate`, vidée seulement dans `flushQueue()` à la ligne 498-506) n'est ni vidée ni son `requestAnimationFrame` annulé. Scénario vérifié dans le code : un `GraphUpdate` pousse dans `_queue` et programme un rAF ; si un `GraphSnapshot` arrive avant l'exécution de ce rAF (ex. ouverture d'un nouveau fichier juste après l'arrêt d'une capture), `loadFromGraphData` appelle `resetGraph()` puis recharge le nouveau snapshot — mais le rAF déjà programmé s'exécute ensuite et rejoue l'update périmé sur le graphe fraîchement chargé (nœud/arête qui n'appartient pas au nouveau relevé).
**Correctif** : dans `resetGraph()`, vider `this._queue` et annuler `this._raf` s'il est programmé.

### 4. `src/components/AnalyseView/panels/ArbitrationDialog.vue:83-102` — échec d'arbitrage jamais montré à l'utilisateur
```ts
} catch (err) {
  error(`Erreur arbitrage: ${err}`);
} finally {
  this.resolving = false;
}
```
Contrairement à toutes les autres mutations de labels de la codebase, aucun `message()`/`displayCaptureError` n'est appelé — `displayCaptureError` n'est même pas importé dans ce fichier. Si le backend refuse l'arbitrage (ex. clé supprimée entre-temps), le bouton se réactive et la boîte reste ouverte sans aucune indication d'échec ; l'utilisateur peut croire l'arbitrage appliqué en fermant la boîte.

### 5. `src/components/AnalyseView/panels/ImportPanel.vue:503-517` — invokes d'arbitrage sans `try/catch`
`onArbitrationResolved()` et `openArbitration()` appellent `get_label_rows`/`get_label_conflicts` sans filet, contrairement à `mounted()` (ligne 529-531) qui catch explicitement le même appel. Si le backend est en défaut au clic sur « Arbitrer les conflits », le dialogue ne s'ouvre jamais et rien n'indique pourquoi ; après un arbitrage réussi, un échec du refresh laisse la table de labels silencieusement périmée.

### 6. `src/components/NavBar/TopBar.vue:119-152` — `export_logs` sans aucun filet d'erreur
`withImportLock` fait seulement `try { await action() } finally { ... }`, sans `catch`. `export_logs()` lève volontairement en cas d'annulation de la boîte de sauvegarde (ligne 146) et ne catch pas non plus les échecs `invoke`. Contrairement à `SaveAsCsv`/`SaveLabels` qui ont chacun leur `try/catch` + `displayCaptureError`. Résultat : annulation de la boîte de dialogue ou échec d'écriture (disque plein, permission refusée) → rejection de promesse non gérée, `isImporting` bien remis à `false` (le verrou ne reste pas bloqué) mais **aucun message n'est montré** ; l'utilisateur croit l'export réussi.

---

## Findings — sévérité moyenne

| # | Fichier:ligne | Problème | Scénario |
|---|---|---|---|
| 7 | `ImportPanel.vue:307-355` vs `:377-413` | Nettoyage incohérent après échec : `packetFiles` vidé inconditionnellement même en erreur, `matrixFiles` vidé seulement en succès | Import de 5 PCAP, le 3ᵉ échoue (link-type non supporté) → toute la sélection est perdue au lieu de ne retirer que le fichier fautif |
| 8 | `ImportPanel.vue:463-464, 497, 508-509` | `filteredlabelRows = labelRows` sans repasser par `filterLabelRows(..., searchInput)` après import/clear/arbitrage | Champ de recherche affiche « router » mais la table montre soudain toutes les lignes après un réimport CSV |
| 9 | `Filter.vue:239-241, 261-263, 270-272` | Échecs `set_filter` catchés en `console.error` brut (plugin-log même pas importé dans ce fichier) — régression de la convention introduite par 93db941d | Rejet backend de `set_filter` → clic « Appliquer » semble ne rien faire, rien dans les logs applicatifs |
| 10 | `Filter.vue:198-212` | `canApply` dépend des erreurs du formulaire guidé même en mode aperçu BPF manuel | IP invalide saisie puis abandonnée + BPF manuel valide tapé → bouton Appliquer reste désactivé sans lien visible avec la saisie réelle |
| 11 | `ConfigPanel.vue:16-66` | `<label>` n'englobe pas l'`<input>`, pas de `for`/`id` | Clic sur le texte du label ne focus pas le champ ; lecteur d'écran annonce un spinbutton sans nom |
| 12 | `NetworkGraphComponent.vue:483-495` + `exportPng.ts:7-17` | `downloadPng()` sans `try/catch`, contrairement à `editNodeLabel()` qui catch et affiche l'erreur | Échec `writeFile` (disque plein) ou `toBlob` → aucun message, l'utilisateur croit l'export réussi |
| 13 | `NetworkGraphComponent.vue:215-261` | `nodeReducer`/`edgeReducer` allouent `{ ...data }` à chaque frame de rendu, même sans interaction (FA2 tourne en continu par défaut, `forceEnabled: true`) | Sur un graphe de plusieurs milliers de nœuds issu d'un import PCAP volumineux, pression GC inutile en continu |
| 14 | `graph/graphSync.ts` (`spawnAnchor`, `upsertNode`) | Recalcul O(n) du barycentre à chaque nouveau nœud → O(n²) cumulé sur une capture live longue | Capture accumulant des milliers d'hôtes : dégradation progressive perceptible |
| 15 | `graph/graphSync.ts` (`updateNodeTrafficSize`) | Reparcourt toutes les arêtes du nœud à chaque upsert d'arête touchant ce nœud → O(degré²) cumulé | Nœud « hub » (passerelle, DNS très sollicité) en capture live : coût cumulé élevé |
| 16 | `graph/forceLayout.ts:45-47` | `resume()` ne relance pas `inferSettings` — réglages FA2 potentiellement obsolètes si le graphe a grossi pendant la pause | Gravité désactivée sur petit graphe, capture continue en tâche de fond, réactivation sur graphe désormais grand → physique instable |
| 17 | `graph/MatrixLabelsPanel.vue:31-38` | Échec `get_matrix_labels` catché en `console.error` puis affiché comme « aucun label » — indiscernable d'un succès vide | Backend indisponible/verrou empoisonné → l'utilisateur croit à tort qu'aucun label n'est appliqué |
| 18 | `TopBar.vue:120-123, 205-208, 226-229, 282-284` | Verrous d'action busy (`isImporting`) : retour silencieux (`stop()`) ou juste un `info()` en log, jamais un message visible | Clic sur Arrêter/Réinitialiser pendant un import en cours → rien ne se passe visuellement, perçu comme un bug |
| 19 | `status-bar/StatusBar.vue:156` | `console.error` au lieu de `@tauri-apps/plugin-log` (fichier n'importe même pas le plugin) — régression convention | Échec `set_filter` en clear devient introuvable en diagnostic support (release) |
| 20 | `status-bar/Cpu.vue:30-53` | Assignation de `this.unlisten` dans le `.then()` de `listen()` (asynchrone) — si démonté avant résolution, le listener réel n'est jamais désinscrit | Montage/démontage rapide (hot-reload, changement de route) → listener `cpu_usage_update` fantôme actif indéfiniment |
| 21 | `.github/workflows/covecode.yml:85` | `deno test --coverage=coverage/frontend src/tests/*.test.js` ne matche que `dateUtils.test.js` (seul `.test.js` du dossier) sur 21 fichiers de test | Le badge Codecov frontend ne reflète que 3 tests sur ~163 ; couverture store/graphe/erreurs IPC/CSP entièrement absente du rapport malgré des tests existants et verts en local |
| 22 | Couverture composants Vue | Zéro test sur les 18 fichiers `.vue` (dont `ImportPanel.vue` 969 lignes, `NetworkGraphComponent.vue` 723 lignes) — seule la logique pure extraite (`graph/*.ts`, `panels/import/*.ts`) est testée | Angle mort déjà identifié en interne ; les régressions listées ci-dessus (ex. #7, #8, #12) ne sont pas détectables par la suite actuelle |

---

## Findings — sévérité basse

- **Modales sans sémantique d'accessibilité** (pattern répété, à traiter globalement plutôt qu'au cas par cas) : `ImportPanel.vue`, `Filter.vue`, `ConfigPanel.vue`, `ConflictDialog.vue`, `graph/MatrixLabelsPanel.vue` — aucune n'a `role="dialog"`/`aria-modal`/fermeture Échap/piège de focus, contrairement à `LabelsPanel.vue` et à la gestion `Escape` de `NetworkGraphComponent.vue` (édition de label) qui montrent le pattern correct déjà en place ailleurs dans l'app.
- `ImportPanel.vue:178,551-552` — `unsubs` déclaré et bouclé au démontage mais jamais alimenté : boucle de nettoyage no-op.
- `ImportPanel.vue:72` — champ de recherche sans `aria-label`.
- `ImportPanel.vue:311` — `convertPcap()` sans le garde `if (!this.isRunning)` présent dans `importMatrixFiles`/`importLabelFile` (inatteignable aujourd'hui car TopBar désactive déjà le bouton, mais incohérence latente).
- `ConfigPanel.vue:26,39,52,65` — `min`/`max`/`step` décoratifs : les champs ne sont pas dans un `<form>`, validation HTML5 jamais déclenchée.
- `ConfigPanel.vue:206-209` + `CustomSelector/interfaceSelector.vue:124-140` — course latente entre auto-sélection d'interface et chargement de la config persistée (fenêtre étroite, se corrige seule).
- `LabelsPanel.vue:255` — `console.warn` au lieu de `plugin-log` (import déjà présent dans le fichier, incohérence ponctuelle).
- `ConflictDialog.vue:23` — `<img>` d'avertissement sans `alt`.
- `ConflictDialog.vue:256-269,285-310` — CSS mort dupliqué depuis `ImportPanel.vue`.
- `NetworkGraphComponent.vue:211,316` — `zoomLevel` réassigné à chaque frame de caméra sans throttle (re-render Vue à chaque frame de zoom/pan).
- `NetworkGraphComponent.vue:550-557` — bouton bascule gravité sans `aria-pressed`.
- `NetworkGraphComponent.vue:174` — `new Graph(...)` non paramétré en generics → `any` implicite propagé dans `nodeInfo.ts`/`labelEdit.ts`/`tunnelHighlight.ts`/`graphSync.ts` sur des champs sensibles (`mac`, `ip`, `encapIds`, `total_bytes`).
- `graph/tunnelHighlight.ts:15-38` — scan complet du graphe à chaque survol d'arête tunnel (perceptible sur graphe dense).
- `BottomLong.vue:148-153` — branche `else` morte dans `beforeUnmount`.
- `BottomLong.vue:73-75,137,148` — cast `(this as unknown as ComponentWithResetBus)` pour `$bus` alors que `types/global.d.ts` le déclare déjà et que `NetworkGraphComponent.vue` y accède directement sans cast — incohérence de style entre deux fichiers voisins.
- `TopBar.vue:13,202-219` — bouton Réinitialiser non désactivé pendant une capture (`isRunning` non vérifié, seulement `isImporting`) ; le backend refuse proprement mais l'UX est incohérente avec les autres boutons.
- `TopBar.vue:15` — `alt="Flux"` copié-collé sur le bouton Config (devrait décrire « Configuration »), identique à l'`alt` du bouton Démarrer → lecteurs d'écran ne distinguent pas les deux boutons.
- `TopBar.vue` — aucun bouton emoji (`🛑🔄💾🏷️🗂️📄❎📒🔍`) n'a d'`aria-label` ; le `title` n'est utilisé comme nom accessible que si le contenu est vide, ce qui n'est pas le cas ici.
- `TopBar.vue:80,67-69,301-303,310-312` — code mort résiduel du refactor #e0ce7da7 : `showMatrice` (data), `toggleView()`, `toggleConfig()`.
- `status-bar/Cpu.vue:42,48` — `warn(msg, extraArg)`/`error(msg, extraArg)` : le plugin-log n'accepte pas de second argument libre, la valeur utile (erreur réelle, valeur CPU invalide) est silencieusement perdue.
- `utils/bpf.ts:57-58` — `isIp`/`isCidr` acceptent des octets hors [0-255] (`999.999.999.999`) ; sans impact réel car le backend revalide à `start_capture` avec message affiché.
- `store/capture.ts` — `CaptureStatus` retypé à la main (`status as {is_running, session_id}`) car non exporté par ts-rs ; un renommage côté Rust ne casserait pas la compilation TS, juste silencieusement `undefined` à l'exécution (risque atténué car `sessionId` est aussi mis à jour via l'event `started`).
- `types/capture.ts:19` — réexport mort de `PacketFlow`, non utilisé ailleurs dans le frontend.
- `eslint.config.js` — `@typescript-eslint/no-explicit-any` désactivé globalement (impact limité, `tsconfig` a `strict: true` qui bloque déjà le `any` implicite).
- `src/tests/**` exclu à la fois du lint et du typecheck strict — écart de rigueur assumé mais réel entre code test et code prod.
- `reproBuildPolicy.test.ts`, `windowsOfficialBuilder.test.ts` — tests par substring/regex sur du texte de script plutôt qu'exécution réelle (build Windows/HSM impossible en CI) ; compromis documenté mais donne une fausse confiance de couverture.
- `utils/appExit.ts` (garde anti-double-clic fermeture) — aucun test malgré un pattern `mockIPC` directement réutilisable dans la suite existante.

---

## Points vérifiés sans anomalie (pour éviter de re-creuser)

- Pas de `v-html`/`innerHTML` nulle part dans le périmètre — aucun vecteur XSS identifié, tous les labels utilisateur/PCAP passent par interpolation Vue échappée ou `canvas.fillText`.
- `LabelsPanel.vue` : aucune mise à jour optimiste avant confirmation backend (`invoke` toujours avant mutation locale) — pattern correct, cohérent avec le fix #161.
- `labelEdit.ts` : rollback défensif correct (vérifie `graph.hasNode` avant réapplication).
- `LegendComponent.vue` : `aria-expanded`/`aria-controls`/`aria-label` corrects, listener `MediaQueryList` bien désabonné.
- `BottomLong.vue` : debounce/plafonnement des lignes de log correct (`MAX_LOG_ROWS`, `flushTimer`), pas de souci perf.
- `ChannelStatus.vue`, `Timer.vue`, `InterfaceStatus.vue` : cleanup listeners/`setInterval` correct, gestion d'erreur présente, pas de `any`/cast dangereux.
- `store/capture.ts` : switch exhaustif sur `CaptureEvent` avec filet `logUnknownEvent(event: never)` bien conçu ; logique de filtrage par `sessionId`/`lastStoppedSessionId` correcte et testée (`captureStore.test.ts`).
- `types/NetDevice.ts` correspond exactement au DTO Rust (`src-tauri/src/dto/mod.rs`), malgré l'absence de génération ts-rs pour ce type.
- `router/*`, `App.vue`, `main.ts` : rien de significatif — plus de routes mortes (nettoyées par 415298c7).
- `import/fileTypes.ts`, `importLifecycle.ts`, `importProgress.ts`, `labelSearch.ts` : petites fonctions pures correctes, pas de `any`, pas de cast dangereux.
- `Filter.vue:239-241` (validation BPF) : le backend ne valide `set_filter` qu'à `start_capture` (message bien affiché là) — pas de "capture cassée silencieusement", juste l'incohérence de logging notée en #9.
- `sonarcube.yml` (contrairement à `covecode.yml`) couvre bien `src/tests/` en entier.
- Écarté après vérification backend : pas de risque de collision de clé d'arête `${source}__${target}__${label}` dans `graphStyle.ts` — `Node.id` est un compteur atomique numérique côté Rust, `Edge.label` un enum de protocole fixe, aucun des deux ne peut contenir `__`.

---

## Ordre de correction recommandé

1. **Erreurs qui font disparaître le message utilisateur en silence** (haute, groupe cohérent) : #1, #2 (gardes défensives manquantes sur payloads imbriqués), #4, #5, #6 (invokes/dialogues sans filet) — c'est le thème dominant de cette revue : plusieurs endroits laissent l'utilisateur sans aucune indication qu'une opération a échoué, ce qui est particulièrement grave pour un outil dont la promesse repose sur la fidélité et la traçabilité du relevé.
2. #3 (queue graphe non vidée au reset) — corruption visuelle silencieuse du graphe affiché, correctif d'une ligne.
3. Régressions de convention logging (#9, #19, LabelsPanel:255) — faciles à corriger, cassent le diagnostic support en release.
4. #21 (Codecov frontend quasi vide) — un badge trompeur ; corriger le glob est trivial (`src/tests/*.test.ts`) et redonnerait une mesure réelle immédiatement.
5. Le reste (moyenne/basse) est du confort UX, de l'a11y et de la dette de performance sur de gros graphes — à prioriser selon la taille réelle des relevés traités en production.
