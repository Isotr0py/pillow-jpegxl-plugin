use std::env;

fn main() {
    #[cfg(not(feature = "vendored"))]
    {
        println!("cargo:rustc-link-lib=static=jxl");
        println!("cargo:rustc-link-lib=static=brotlidec-static");
        println!("cargo:rustc-link-lib=static=jxl_threads");
        println!("cargo:rustc-link-lib=static=brotlidec-static");

        println!("cargo:rustc-link-lib=static=hwy");
        println!("cargo:rustc-link-lib=static=brotlidec-static");
        if let Ok(path) = env::var("DEP_HWY_LIB") {
            println!("cargo:rustc-link-search=native={}", path);
        }

        println!("cargo:rustc-link-lib=static=brotlidec-static");
        println!("cargo:rustc-link-lib=static=brotlienc-static");
        println!("cargo:rustc-link-lib=static=brotlicommon-static");
        if let Ok(path) = env::var("DEP_BROTLI_LIB") {
            println!("cargo:rustc-link-search=native={}", path);
        }
        println!("cargo:rustc-link-lib=static=brotlidec-static");

    }

    println!("cargo:rustc-link-lib=stdc++");
    // println!("cargo:rerun-if-changed=build.rs");
}