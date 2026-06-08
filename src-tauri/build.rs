fn main() {
    println!("cargo:rerun-if-changed=../config/build-versions.env");

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

    tauri_build::build();

    // Only add the Packet.lib library on Windows
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-search=native=./lib");
        println!("cargo:rustc-link-lib=static=Packet");
        println!("cargo:rustc-link-lib=static=wpcap");
    }
}
