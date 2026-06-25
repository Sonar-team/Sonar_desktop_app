# Sprint: performance capture sous forte charge réseau

## Objectif
Réduire la pression CPU et IPC pendant la capture live, afin de garder une
UI réactive même quand le débit réseau est élevé.

## Constat
- Le frontend est déjà partiellement protégé par du batching et du throttling.
- Le backend reste le point le plus coûteux sur le chemin critique.
- Les mises à jour de matrice et de graphe sont encore trop proches de la
  réception paquet par paquet.

## Périmètre du sprint
- Diminuer le coût par paquet dans le thread de traitement.
- Limiter les locks et les recalculs sur `FlowMatrix` et `GraphData`.
- Préparer une stratégie de dégradation contrôlée quand la charge augmente.
- Garder les stats et l’état de capture fiables.

## Priorités
1. Mesurer les coûts réels par étape du pipeline de capture.
2. Batcher les mises à jour backend internes sur la matrice et le graphe.
3. Réduire le nombre de verrouillages mutex dans le traitement des paquets.
4. Décorréler les updates graphiques du chemin critique de capture.
5. Définir une politique de backpressure et de dégradation progressive.

## Focus technique
### Priorité 1: alléger le thread de traitement
- Aujourd’hui, le thread de traitement fait trop de choses à la suite pour
  chaque paquet.
- Le refactor visé consiste à le découper en deux niveaux:
  - chemin critique minimal: parsing léger, comptage, push dans une file;
  - worker(s) différés: matrice, graphe, enrichissement label, événements UI.
- But: éviter que la capture bloque parce que la matrice ou le graphe prennent
  trop de temps.

### Priorité 2: batcher aussi la matrice et le graphe
- Le batching existe déjà pour l’envoi frontend, mais pas encore vraiment pour
  la logique interne.
- Les mises à jour de `FlowMatrix` et `GraphData` doivent être regroupées par
  fenêtre de temps ou par lot de N paquets.
- Cible de départ:
  - 50 ms ou 256 paquets;
  - puis une seule mise à jour agrégée.
- Gains attendus:
  - moins de locks mutex;
  - moins de recalculs de labels;
  - moins de `GraphUpdate`.

## Livrables attendus
- Profiling minimal du pipeline de capture.
- Refactor du thread de traitement pour isoler le travail lourd.
- Mise à jour plus robuste du graphe sous forte charge.
- Note de validation sur le comportement en surcharge.

## Fichiers candidats
- `src-tauri/src/state/capture/capture_handle/threads/processing.rs`
- `src-tauri/src/state/capture/capture_handle/threads/capture.rs`
- `src-tauri/src/state/capture/capture_handle/mod.rs`
- `src/components/AnalyseView/NetworkGraphComponent.vue`
- `src/components/AnalyseView/BottomLong.vue`

## Risques
- Réduction excessive des updates visibles si le batching est trop agressif.
- Dégradation de la précision du graphe si les deltas sont trop espacés.
- Contention persistante si la matrice reste mise à jour paquet par paquet.

## Plan d'exécution par fichier
### `src-tauri/src/state/capture/capture_handle/threads/capture.rs`
- Garder le thread de capture strictement orienté acquisition.
- Limiter les responsabilités à:
  - lecture packet;
  - copie dans un buffer pool;
  - push vers la file applicative;
  - remontée de backpressure si saturation.

### `src-tauri/src/state/capture/capture_handle/threads/processing.rs`
- Extraire la logique lourde dans un worker différé ou une étape batchée.
- Réduire les `lock()` successifs sur `FlowMatrix`.
- Regrouper la mise à jour de `GraphData` par fenêtre temporelle.
- Émettre les événements UI par lot plutôt qu’un par paquet.

### `src-tauri/src/state/capture/capture_handle/mod.rs`
- Ajuster l’orchestration des threads pour intégrer le nouveau découpage.
- Prévoir les canaux ou files nécessaires au worker différé.
- Conserver le comportement d’arrêt propre avec `stop_flag`.

### `src/components/AnalyseView/NetworkGraphComponent.vue`
- Vérifier que le graphe supporte mieux des updates moins fréquentes mais plus
  volumineuses.
- Conserver le rendu fluide avec `requestAnimationFrame`.
- Éviter les reconstructions complètes inutiles quand seul un delta suffit.

### `src/components/AnalyseView/BottomLong.vue`
- Maintenir le throttling actuel.
- Vérifier que le log reste lisible si le backend envoie des paquets par lot
  plus grands.

## Critères d'acceptation
- La capture reste stable sous forte charge sans bloquer le traitement.
- Le thread de capture ne porte plus la logique métier lourde.
- Les mises à jour de matrice et de graphe sont batchées.
- La UI reste réactive, même si les rafraîchissements sont moins fréquents.
- Les stats et l’état de backpressure restent cohérents.

## Tâches de sprint
### SP-01 - Mesurer le pipeline de capture
- Ajouter des mesures de temps sur le parsing, la matrice, le graphe et l'IPC.
- Objectif: identifier le vrai goulet d'étranglement avant refactor.
- Pour le parsing, utiliser la feature `capture_timing` et écrire les mesures
  dans un fichier JSONL, sans passer par l'IPC frontend.
- Étendre la même ligne de mesure aux phases connexes du chemin critique:
  conversion en paquet owned, lookup labels, update `FlowMatrix`, update
  `GraphData`, envoi IPC des updates graphe.
- Ajouter une mesure séparée pour le flush IPC des `PacketBatch`.
- Pour les relevés reproductibles hors capture live, instrumenter aussi
  l'import PCAP avec le même fichier rejoué plusieurs fois.
- Les événements d'import attendus dans le JSONL sont:
  `import_packet_timing`, `import_parse_error_timing`, `import_file_timing`
  et `import_snapshot_timing`.
- Les lignes `import_file_timing` doivent exposer `parse_ok` et
  `parse_errors`, afin de distinguer le coût des paquets réellement traités
  du coût des paquets lus mais non supportés par le parser.
- Les fichiers de profiling doivent être ingérés par un collecteur externe
  vers Grafana/Loki ou PostgreSQL, afin de ne pas ajouter de charge réseau ou
  base de données dans le thread de capture.

## Protocole de relevé par import PCAP
- Compiler/lancer avec la feature `capture_timing`.
- Fixer `SONAR_CAPTURE_TIMING_LOG` pour écrire toutes les mesures dans le même
  fichier JSONL.
- Pour les petits PCAP, garder `SONAR_IMPORT_TIMING_SAMPLE_RATE=1`.
- Réimporter plusieurs fois le même fichier PCAP et comparer les lignes
  `import_file_timing` et `import_packet_timing`.
- Sur les gros PCAP, vérifier aussi `parse_ok`, `parse_errors` et les lignes
  `import_parse_error_timing` pour ne pas conclure uniquement sur les paquets
  que le parser accepte.

### SP-02 - Séparer le chemin critique de capture
- Réduire `capture.rs` au strict minimum opérationnel.
- Ne garder que la lecture, la copie en buffer pool et l'envoi vers la file.

### SP-03 - Introduire un worker différé pour le traitement lourd
- Déplacer la mise à jour de `FlowMatrix` et `GraphData` hors du chemin critique.
- Prévoir une file ou un lot de travail dédié.

### SP-04 - Batcher les mises à jour de matrice
- Regrouper les updates de `FlowMatrix` par fenêtre temporelle ou par lot.
- Limiter les verrous successifs et les recalculs répétés.

### SP-05 - Batcher les mises à jour de graphe
- Regrouper les `GraphUpdate` avant émission.
- Préserver le rendu sans envoyer un événement par paquet.

### SP-06 - Valider la résistance UI sous forte charge
- Vérifier le comportement de `NetworkGraphComponent.vue`.
- Vérifier que `BottomLong.vue` reste stable avec des lots plus gros.

### SP-07 - Définir la stratégie de dégradation
- Déterminer ce qui doit être conservé en priorité en cas de saturation.
- Formaliser le comportement de backpressure et de throttling.
