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
sonar-cli graph merged.csv -o network.svg \
  --min-bytes 10000 --max-nodes 200 --protocol TCP --labels ip
```

The graph command accepts `.dot`, `.svg`, and `.png` outputs. SVG and PNG
rendering requires Graphviz; `sfdp` is the default layout engine and can be
changed with `--engine`.
