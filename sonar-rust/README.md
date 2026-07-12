# Sonar Rust Workspace

This workspace hosts the shared Sonar domain logic and its terminal binary.
The Tauri application in `src-tauri/` depends on `sonar-flows-core`, so any
domain fix belongs here, once, rather than in the desktop app.

Current layout:

- `crates/sonar-flows-core`: reusable Rust library, with no Tauri dependency.
- `crates/sonar-flows-cli`: Clap-based command-line binary (installed as
  `sonar-cli`) using `sonar-flows-core`.

Command shape:

```sh
sonar-cli pcap ezra.pcap -o ezra.csv
sonar-cli matrix matrice-a.csv matrice-b.csv -o merged.csv
```
