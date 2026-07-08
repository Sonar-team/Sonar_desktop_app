# Sonar Rust Workspace

This workspace is intentionally isolated from the current Tauri application in
`src-tauri/`.

It is the staging area for extracting reusable Sonar domain logic into
`sonar-core` and building a dedicated terminal binary in `sonar-cli`.

Current layout:

- `crates/sonar-core`: reusable Rust library, with no Tauri dependency.
- `crates/sonar-cli`: Clap-based command-line binary using `sonar-core`.

Target command shape:

```sh
sonar-cli pcap ezra.pcap -o ezra.csv
sonar-cli matrix matrice-a.csv matrice-b.csv -o merged.csv
```
