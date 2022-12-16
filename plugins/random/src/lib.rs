#![deny(unsafe_code)]

use ayaka_bindings::*;

#[export]
fn plugin_type() -> PluginType {
    PluginType::default()
}

#[import("rand")]
extern "C" {
    fn __rnd(start: i64, end: i64) -> i64;
}

#[export]
fn rnd(args: Vec<RawValue>) -> RawValue {
    let res = match args.len() {
        0 => __rnd(0, i64::MAX),
        1 => __rnd(0, args[0].get_num()),
        _ => __rnd(args[0].get_num(), args[1].get_num()),
    };
    RawValue::Num(res)
}
