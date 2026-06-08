pub fn about_message() -> String {
    format!(
        "SONAR {}\n\nRust: {}\nNode.js: {}\nDeno: {}\nTauri CLI: {}",
        env!("CARGO_PKG_VERSION"),
        option_env!("SONAR_RUST_VERSION").unwrap_or("unknown"),
        option_env!("SONAR_NODE_VERSION").unwrap_or("unknown"),
        option_env!("SONAR_DENO_VERSION").unwrap_or("unknown"),
        option_env!("SONAR_TAURI_CLI_VERSION").unwrap_or("unknown"),
    )
}
