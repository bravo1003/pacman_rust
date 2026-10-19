//! Links clang's runtime library into static macOS builds.
//!
//! The bundled SDL2 is compiled for an older macOS than the SDK, so its
//! `@available` checks call `__isPlatformVersionAtLeast`. That function lives
//! in clang's runtime library, which rustc does not link by itself.

use std::env;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let static_sdl = env::var_os("CARGO_FEATURE_STATIC").is_some();
    if !static_sdl || env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }
    let output = Command::new("clang")
        .arg("--print-runtime-dir")
        .output()
        .expect("clang is required to build the bundled SDL2");
    let runtime_dir = String::from_utf8(output.stdout).expect("clang printed a non-UTF-8 path");
    println!("cargo:rustc-link-search=native={}", runtime_dir.trim());
    println!("cargo:rustc-link-lib=static=clang_rt.osx");
}
