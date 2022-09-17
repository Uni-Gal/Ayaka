use ayaka_bindings::*;

#[export]
fn plugin_type() -> PluginType {
    PluginType::default()
}

#[import("format")]
extern "C" {
    fn __format(args: Vec<RawValue>) -> RawValue;
}

#[export]
fn fmt(args: Vec<RawValue>) -> RawValue {
    __format(args)
}
