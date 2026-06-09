fn main() {
    println!("cargo:rerun-if-changed=../config/build-versions.env");
    println!("cargo:rerun-if-changed=windows/npcap-sdk/Lib/x64/Packet.lib");
    println!("cargo:rerun-if-changed=windows/npcap-sdk/Lib/x64/wpcap.lib");

    let build_versions = std::fs::read_to_string("../config/build-versions.env")
        .expect("failed to read ../config/build-versions.env");

    for line in build_versions.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        let env_key = match key.trim() {
            "RUST_VERSION" => "SONAR_RUST_VERSION",
            "NODE_VERSION" => "SONAR_NODE_VERSION",
            "DENO_VERSION" => "SONAR_DENO_VERSION",
            "TAURI_CLI_VERSION" => "SONAR_TAURI_CLI_VERSION",
            _ => continue,
        };

        println!(
            "cargo:rustc-env={env_key}={}",
            value.trim().trim_matches('"')
        );
    }

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rustc-link-search=native=windows/npcap-sdk/Lib/x64");
        println!("cargo:rustc-link-lib=Packet");
        println!("cargo:rustc-link-lib=wpcap");
    }

    tauri_build::build();
}
