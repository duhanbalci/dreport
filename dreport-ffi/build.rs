//! Generates `include/dreport.h` from the public C ABI on every build.
//! Header is checked into the repo so consumers (NuGet wrapper, manual C use)
//! don't need a Rust toolchain.

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");
    println!("cargo:rerun-if-changed=build.rs");

    // Skip generation if explicitly disabled (e.g. when cross-compiling without
    // host tools, or in cargo publish dry runs).
    if env::var("DREPORT_FFI_SKIP_CBINDGEN").is_ok() {
        return;
    }

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_path = PathBuf::from(&crate_dir).join("include").join("dreport.h");
    if let Some(parent) = out_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let config_path = PathBuf::from(&crate_dir).join("cbindgen.toml");
    let config = if config_path.exists() {
        cbindgen::Config::from_file(&config_path).expect("invalid cbindgen.toml")
    } else {
        cbindgen::Config::default()
    };

    match cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(config)
        .generate()
    {
        Ok(bindings) => {
            bindings.write_to_file(&out_path);
        }
        Err(e) => {
            // Don't fail the build on header generation problems — the cdylib
            // is still usable; the header is a developer convenience.
            println!("cargo:warning=cbindgen header generation failed: {}", e);
        }
    }
}
