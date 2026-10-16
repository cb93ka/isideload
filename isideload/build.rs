fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rerun-if-changed=src/anisette/macos_local.m");
        cc::Build::new()
            .file("src/anisette/macos_local.m")
            .flag("-fobjc-arc")
            .compile("isideload_macos_local");
        println!("cargo:rustc-link-lib=framework=Foundation");
    }
}
