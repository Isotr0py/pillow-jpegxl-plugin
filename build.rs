use std::env;

fn dynamic_link() {
    println!("cargo:rustc-link-lib=jxl");
    println!("cargo:rustc-link-lib=jxl_threads");

    println!("cargo:rustc-link-lib=hwy");
    if let Ok(path) = env::var("DEP_HWY_LIB") {
        println!("cargo:rustc-link-search=native={}", path);
    }

    println!("cargo:rustc-link-lib:+whole-archive=brotlidec");
    println!("cargo:rustc-link-lib=brotlienc");
    println!("cargo:rustc-link-lib=brotlicommon");
    if let Ok(path) = env::var("DEP_BROTLI_LIB") {
        println!("cargo:rustc-link-search=native={}", path);
    }
}

fn static_link() {
    println!("cargo:rustc-link-lib=static=jxl");
    println!("cargo:rustc-link-lib=static=jxl_cms");
    println!("cargo:rustc-link-lib=static=jxl_threads");

    println!("cargo:rustc-link-lib=static=hwy");
    if let Ok(path) = env::var("DEP_HWY_LIB") {
        println!("cargo:rustc-link-search=native={}", path);
    }

    println!("cargo:rustc-link-lib=static:+whole-archive=brotlidec");
    println!("cargo:rustc-link-lib=static=brotlienc");
    println!("cargo:rustc-link-lib=static=brotlicommon");
    if let Ok(path) = env::var("DEP_BROTLI_LIB") {
        println!("cargo:rustc-link-search=native={}", path);
    }
}

fn main() {
    // Static link libjxl
    #[cfg(all(not(feature = "vendored"), not(feature = "dynamic")))]
    static_link();

    #[cfg(all(not(feature = "vendored"), feature = "dynamic"))]
    dynamic_link();

    // Dynamic link c++
    // TODO: Support MSVC and use Cargo
    let platform = env::consts::OS;
    match platform {
        // Since MSVC will stuck on building libjxl
        // Linux and Windows should all use GNU toolchain
        "linux" | "windows" => println!("cargo:rustc-link-lib=stdc++"),
        "macos" => println!("cargo:rustc-link-lib=c++"),
        _ => panic!("Not implemented c++ link on {}", platform),
    }
}
