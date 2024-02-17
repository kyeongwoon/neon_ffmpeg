use std::{env, path::PathBuf};

fn main() {
    let _target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    //println!("cargo:warning=target_os: {}", target_os);

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    println!(r"cargo:rustc-link-search=/opt/homebrew/lib");

    #[cfg(not(target_arch = "aarch64"))]
    println!(r"cargo:rustc-link-search=/usr/local/lib");

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    println!("cargo:include={}", "/opt/homebrew/include");

    #[cfg(not(target_arch = "aarch64"))]
    println!("cargo:include={}", "/usr/local/include");

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    println!("cargo:rustc-env=PREFIX={}", "/opt/homebrew");

    #[cfg(not(target_arch = "aarch64"))]
    println!("cargo:rustc-env=PREFIX={}", "/usr/local");

    /*
        println!("cargo:rustc-link-lib=dylib=avcodec");
        println!("cargo:rustc-link-lib=dylib=avformat");
        println!("cargo:rustc-link-lib=dylib=avutil");
        println!("cargo:rustc-link-lib=dylib=avdevice");
        println!("cargo:rustc-link-lib=dylib=avpostproc");
        println!("cargo:rustc-link-lib=dylib=avfilter");
        println!("cargo:rustc-link-lib=dylib=swresample");
        println!("cargo:rustc-link-lib=dylib=swscale");
        println!("cargo:rustc-link-lib=dylib=ass");
    */
    //println!("cargo:warning=This is a custom build script!");
}
