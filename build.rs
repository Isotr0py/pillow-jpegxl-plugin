fn main() {
    #[cfg(not(feature = "vendored"))]
    {
        println!("cargo:rustc-link-lib=static=brotlidec-static");
        println!("cargo:rustc-link-lib=static=brotlienc-static");
        println!("cargo:rustc-link-lib=static=brotlicommon-static");

        println!("cargo:rustc-link-lib=static=hwy");

        println!("cargo:rustc-link-lib=static=jxl_threads");
        println!("cargo:rustc-link-lib=static=jxl");
    }

    println!("cargo:rustc-link-lib=stdc++");
}