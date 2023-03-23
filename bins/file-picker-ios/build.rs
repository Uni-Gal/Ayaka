fn main() {
    std::env::set_var("IPHONEOS_DEPLOYMENT_TARGET", "14.0");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=native/picked.m");
    cc::Build::new()
        .file("native/picker.m")
        .flag("-fobjc-arc")
        .flag("-std=c11")
        .compile("picker");
}
