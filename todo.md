  Je ferais sonar-core comme une lib Rust pure, publiable sur crates.io, sans dépendance Tauri :

  sonar-core
    matrix       # FlowMatrix, FlowMatrixRow, merge, export CSV
    pcap         # PCAP/PCAPNG -> FlowMatrix
    labels       # import/export labels, conflits
    graph        # optionnel si considéré domaine métier
    errors       # thiserror, erreurs stables

  Puis :

  sonar-cli         # clap, stdout/stderr, exit codes
  sonar-desktop     # Tauri UI, invoke handlers, windows, events

  Le point important : sonar-core ne doit pas connaître tauri::State, Channel, AppHandle, WebView, menus, logs Tauri, etc. Il doit exposer des fonctions simples :

  let matrix = sonar_core::pcap::convert_files(&paths, options)?;
  sonar_core::csv::write_matrix(&matrix, output)?;

  Pour le desktop, les commandes Tauri deviennent juste des adaptateurs :

  - récupèrent les args depuis l’UI
  - appellent sonar-core
  - traduisent le résultat en events/snapshots UI

  Pour crates.io, je ferais attention à deux choses :

  - feature flags : pcap devrait probablement être une feature, car libpcap/Npcap rend l’installation plus lourde.
  - API stable : ne pas publier trop tôt une API interne brouillonne. D’abord extraire proprement, puis publier quand sonar-cli et sonar-desktop utilisent vraiment la crate.

  Plan pragmatique :

  1. Créer un workspace Cargo.
  2. Extraire FlowMatrix, FlowMatrixRow, CSV import/export dans sonar-core.
  3. Extraire PCAP -> matrix dans sonar-core, avec callback de progression optionnel.
  4. Adapter Tauri à la nouvelle lib.
  5. Créer sonar-cli avec clap.
  6. Publier sonar-core quand l’API est propre.

  Donc oui : sonar-core + sonar-cli + sonar_desktop_app, c’est beaucoup plus sain que d’étendre indéfiniment le headless Tauri.
