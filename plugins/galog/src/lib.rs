use gal_bindings::*;

#[export]
fn plugin_type() -> PluginType {
    PluginType::SCRIPT
}

fn log_impl(level: log::Level, args: Vec<RawValue>) -> RawValue {
    let mut buffer = String::new();
    for arg in args.into_iter() {
        buffer.push_str(&arg.get_str());
    }
    log::log!(level, "{}", buffer);
    RawValue::Unit
}

#[export]
fn error(args: Vec<RawValue>) -> RawValue {
    log_impl(log::Level::Error, args)
}

#[export]
fn warn(args: Vec<RawValue>) -> RawValue {
    log_impl(log::Level::Warn, args)
}

#[export]
fn info(args: Vec<RawValue>) -> RawValue {
    log_impl(log::Level::Info, args)
}

#[export]
fn debug(args: Vec<RawValue>) -> RawValue {
    log_impl(log::Level::Debug, args)
}

#[export]
fn trace(args: Vec<RawValue>) -> RawValue {
    log_impl(log::Level::Trace, args)
}
