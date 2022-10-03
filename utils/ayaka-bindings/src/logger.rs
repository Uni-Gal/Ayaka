use ayaka_bindings_impl::import;
use log::Log;

#[link(wasm_import_module = "log")]
extern "C" {
    fn __log_flush();
}

#[import("log")]
extern "C" {
    fn __log(record: ayaka_bindings_types::Record);
}

pub struct PluginLogger;

impl PluginLogger {
    pub fn init() {
        use std::sync::Once;
        static INIT: Once = Once::new();

        INIT.call_once(|| {
            let r = log::set_logger(&PluginLogger);
            if r.is_ok() {
                log::set_max_level(log::LevelFilter::Trace);
            }
            r.unwrap();
        });
    }
}

impl Log for PluginLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        __log(record.into());
    }

    fn flush(&self) {
        unsafe { __log_flush() }
    }
}
