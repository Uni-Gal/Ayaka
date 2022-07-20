use log::Log;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "log")]
extern "C" {
    fn __log(len: usize, data: *const u8);
    fn __log_flush();
}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn __log(_len: usize, _data: *const u8) {}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn __log_flush() {}

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
        let record: gal_bindings_types::Record = record.into();
        let data = rmp_serde::to_vec(&record).unwrap();
        unsafe { __log(data.len(), data.as_ptr()) }
    }

    fn flush(&self) {
        unsafe { __log_flush() }
    }
}
