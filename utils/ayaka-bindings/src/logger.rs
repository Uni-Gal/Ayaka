use crate::import;
use log::Log;

#[import("log")]
extern "C" {
    fn __log(record: ayaka_bindings_types::Record);
    fn __log_flush();
}

pub struct PluginLogger;

impl PluginLogger {
    pub fn init() {
        use std::sync::Once;
        static INIT: Once = Once::new();

        INIT.call_once(|| {
            log::set_logger(&PluginLogger).expect("cannot set logger");
            log::set_max_level(log::LevelFilter::Trace);
        });
    }
}

impl Log for PluginLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        __log(record.into())
    }

    fn flush(&self) {
        __log_flush()
    }
}
