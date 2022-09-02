use gal_bindings::*;
use log::warn;
use rt_format::*;
use std::collections::HashMap;

#[export]
fn plugin_type() -> PluginType {
    PluginType::default()
}

#[export]
fn fmt(args: Vec<RawValue>) -> RawValue {
    if args.is_empty() {
        warn!("Format args is empty.");
        RawValue::Unit
    } else {
        ParsedFormat::parse(
            &args[0].get_str(),
            &args[1..],
            &HashMap::<String, RawValue>::new(),
        )
        .map(|r| RawValue::Str(r.to_string()))
        .unwrap_or_else(|i| {
            warn!("Format failed, stopped at {}.", i);
            Default::default()
        })
    }
}
