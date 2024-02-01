use std::{env, path::PathBuf};
	
	
fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    println!("cargo:warning=target_os: {}", target_os);


    #[cfg(any(target_os = "macos", target_os = "macos"))]
    println!(r"cargo:rustc-link-search=/opt/homebrew/lib");
    println!("cargo:include={}", "/opt/homebrew/include");

    println!("cargo:rustc-env=PREFIX={}", "/opt/homebrew");
    //println!("cargo:rustc-link-lib=dylib=iconv");
    //println!("cargo:rustc-link-lib=dylib=ass");

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
    // 빌드 과정 중에 사용자 정의 작업 수행
    println!("cargo:warning=This is a custom build script!");
}
