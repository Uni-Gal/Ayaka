fn main() {
    std::env::set_var("STATIC_VCRUNTIME", "false");
    std::env::set_var("PKG_CONFIG_ALLOW_CROSS", "1");
    tauri_build::build()
}
