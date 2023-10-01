fn main() {
    #[cfg(not(feature = "vendored"))]
    {
        println!("cargo:rustc-link-lib=static=brotlidec-static");
        println!("cargo:rustc-link-lib=static=jxl");
        println!("cargo:rustc-link-lib=static=jxl_threads");

        // println!("cargo:rustc-link-lib=hwy");

        // println!("cargo:rustc-link-lib=brotlidec");
        // println!("cargo:rustc-link-lib=brotlienc");
        // println!("cargo:rustc-link-lib=brotlicommon");
    }

    println!("cargo:rustc-link-lib=stdc++");
}