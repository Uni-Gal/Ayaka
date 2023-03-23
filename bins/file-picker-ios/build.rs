fn main() {
    cc::Build::new()
        .file("native/picker.m")
        .flag("-fobjc-arc")
        .flag("-std=c11")
        .compile("picker");
}
