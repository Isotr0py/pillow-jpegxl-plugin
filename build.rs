use std::env;

fn main() {
    // Static link libjxl
    #[cfg(not(feature = "vendored"))]
    {
        println!("cargo:rustc-link-lib=static=jxl");
        println!("cargo:rustc-link-lib=static=jxl_threads");

        println!("cargo:rustc-link-lib=static=hwy");
        if let Ok(path) = env::var("DEP_HWY_LIB") {
            println!("cargo:rustc-link-search=native={}", path);
        }

        println!("cargo:rustc-link-lib=static:+whole-archive=brotlidec-static");
        println!("cargo:rustc-link-lib=static=brotlienc-static");
        println!("cargo:rustc-link-lib=static=brotlicommon-static");
        if let Ok(path) = env::var("DEP_BROTLI_LIB") {
            println!("cargo:rustc-link-search=native={}", path);
        }
    }

    // Dynamic link c++
    // TODO: Support MSVC and use Cargo
    let platform = env::consts::OS;
    match platform {
        // Since MSVC will stuck on building libjxl
        // Linux and Windows should all use GNU toolchain
        "linux" | "windows" => println!("cargo:rustc-link-lib=stdc++"),
        "macos" => println!("cargo:rustc-link-lib=c++"),
        _ => panic!("Not implemented c++ link on {}", platform)
    }
}