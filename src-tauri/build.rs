fn main() {
    // Tauri build process
    tauri_build::build();

    // Only add the Packet.lib library on Windows
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-search=native=./lib");
        println!("cargo:rustc-link-lib=static=Packet");
        println!("cargo:rustc-link-lib=static=wpcap");
    }
}
