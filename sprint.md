# Sprint: Sonar Core and CLI Extraction

## Objectif

Preparer une architecture Rust separee pour extraire le moteur Sonar dans une
crate reutilisable, sans casser l'application Tauri actuelle.

## Livrables

- Workspace Cargo isole dans `sonar-rust/`.
- Crate `sonar-core` sans dependance Tauri.
- Binaire `sonar-cli` base sur `clap`.
- Premier squelette de commandes batch:
  - `sonar-cli pcap <files...> -o <matrix.csv>`
  - `sonar-cli matrix <files...> -o <merged.csv>`

## Plan de travail

1. Stabiliser le workspace `sonar-rust`.
2. Extraire les types de matrice dans `sonar-core`.
3. Extraire l'import/export CSV dans `sonar-core`.
4. Extraire la conversion PCAP vers matrice dans `sonar-core`.
5. Brancher `sonar-cli` sur les fonctions reelles.
6. Adapter progressivement `src-tauri` pour consommer `sonar-core`.
7. Ajouter tests unitaires et tests de non-regression sur les matrices CSV.

## Contraintes

- Ne pas modifier le comportement de l'application desktop pendant la phase
  d'extraction.
- Garder `sonar-core` independant de Tauri, WebView, `Channel`, `State` et
  `AppHandle`.
- Garder des erreurs propres et stables pour que la CLI puisse retourner des
  exit codes fiables.
- Isoler les dependances lourdes comme `pcap` derriere des features si cela
  devient necessaire pour publier sur crates.io.

## Criteres d'acceptation du MVP CLI

- `sonar-cli --help` affiche les commandes disponibles.
- `sonar-cli pcap input.pcap -o output.csv` genere une matrice CSV.
- `sonar-cli matrix a.csv b.csv -o merged.csv` fusionne les matrices.
- Les erreurs d'entree/sortie sont lisibles dans `stderr`.
- Les commandes retournent `0` en succes et un code non nul en erreur.

## Risques

- Le code actuel d'import PCAP est encore couple aux evenements Tauri.
- Le binaire desktop Windows n'est pas ideal comme executable console.
- La publication crates.io demande une API plus stable que les modules internes
  actuels.
