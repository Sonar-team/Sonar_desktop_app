fn main() -> Result<()> {
    webview2_link::update_rustc_flags(webview2_link::output_libs(
        webview2_path::get_manifest_dir()?,
        webview2_path::get_out_dir()?,
    )?)?;

    println!("cargo:rustc-link-lib=advapi32");
    Ok(())
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Var(#[from] std::env::VarError),
}

pub type Result<T> = std::result::Result<T, Error>;

#[macro_use]
extern crate thiserror;

mod webview2_path {
    use std::{convert::From, env, path::PathBuf};

    pub fn get_out_dir() -> super::Result<PathBuf> {
        Ok(PathBuf::from(env::var("OUT_DIR")?))
    }

    pub fn get_manifest_dir() -> super::Result<PathBuf> {
        Ok(PathBuf::from(env::var("CARGO_MANIFEST_DIR")?))
    }
}

mod webview2_link {
    use std::{env, path::PathBuf};

    pub fn output_libs(manifest_dir: PathBuf, out_dir: PathBuf) -> super::Result<PathBuf> {
        const WEBVIEW2_LIBS: &[&str] = &[
            "WebView2Loader.dll",
            "WebView2Loader.dll.lib",
            "WebView2LoaderStatic.lib",
        ];
        const WEBVIEW2_TARGETS: &[&str] = &["arm64", "x64", "x86"];

        for target in WEBVIEW2_TARGETS {
            for lib in WEBVIEW2_LIBS {
                use std::fs;

                let mut lib_src = manifest_dir.clone();
                lib_src.push(target);
                lib_src.push(lib);

                let mut lib_dest = out_dir.clone();
                lib_dest.push(target);
                if !lib_dest.is_dir() {
                    fs::create_dir(lib_dest.as_path())?;
                }

                lib_dest.push(lib);
                eprintln!("Copy from {:?} -> {:?}", lib_src, lib_dest);
                fs::copy(lib_src.as_path(), lib_dest.as_path())?;
            }
        }

        Ok(out_dir)
    }

    pub fn update_rustc_flags(lib_path: PathBuf) -> super::Result<()> {
        let target_arch = match env::var("CARGO_CFG_TARGET_ARCH")?.as_str() {
            "x86_64" => "x64",
            "x86" => "x86",
            "arm" => "arm",
            "aarch64" => "arm64",
            unimplemented => unimplemented!(
                "`{}` architecture set by `CARGO_CFG_TARGET_ARCH`",
                unimplemented
            ),
        };
        let mut lib_path = lib_path;
        lib_path.push(target_arch);

        match lib_path.to_str() {
            Some(path) if lib_path.exists() => println!("cargo:rustc-link-search=native={path}"),
            _ => unimplemented!("`{}` is not supported by WebView2", target_arch),
        };

        Ok(())
    }
}
