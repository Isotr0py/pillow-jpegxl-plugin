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
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=stdc++");
    }
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=c++");
    }
}
