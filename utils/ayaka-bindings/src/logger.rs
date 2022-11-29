use log::Log;

#[link(wasm_import_module = "log")]
extern "C" {
    fn __log(len: usize, data: *const u8);
    fn __log_flush();
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
        let record: ayaka_bindings_types::Record = record.into();
        let record: ayaka_bindings_types::Record = record.into();
        let data = rmp_serde::to_vec(&(record,)).unwrap();
        unsafe { __log(data.len(), data.as_ptr()) };
    }

    fn flush(&self) {
        unsafe { __log_flush() }
    }
}
