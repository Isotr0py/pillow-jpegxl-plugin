use std::env;

fn main() {
    #[cfg(not(feature = "vendored"))]
    {
        println!("cargo:rustc-link-lib=static=jxl");
        println!("cargo:rustc-link-lib=static=jxl_threads");

        println!("cargo:rustc-link-lib=static=hwy");
        if let Ok(path) = env::var("DEP_HWY_LIB") {
            println!("cargo:rustc-link-search=native={}", path);
        }

        // println!("cargo:rustc-link-lib=static=brotlidec-static");
        // println!("cargo:rustc-link-lib=static=brotlienc-static");
        // println!("cargo:rustc-link-lib=static=brotlicommon-static");
        // println!("cargo:rustc-link-search=native={}", "/mnt/d/rust/dev/test/libjxl/build/third_party/brotli");
        println!("cargo:rustc-link-lib=static=brotlicommon-static");
        println!("cargo:rustc-link-lib=static=brotlidec-static");
        println!("cargo:rustc-link-lib=static=brotlienc-static");
        if let Ok(path) = env::var("DEP_BROTLI_LIB") {
            println!("cargo:rustc-link-search=native={}", path);
        }

        // println!("cargo:rustc-link-lib=brotlidec");
        // println!("cargo:rustc-link-lib=brotlienc");
        // println!("cargo:rustc-link-lib=brotlicommon");
    }

    println!("cargo:rustc-link-lib=stdc++");
}