use ayaka_bindings::*;

#[export]
fn plugin_type() -> PluginType {
    PluginType::default()
}

#[link(wasm_import_module = "format")]
extern "C" {
    fn __format(len: usize, data: *const u8) -> u64;
}

#[export]
fn fmt(args: Vec<RawValue>) -> RawValue {
    unsafe { __import(__format, (args,)) }
}
