pub use concat_idents::concat_idents;
pub use gal_primitive::*;
pub use log;

use log::Log;
use std::alloc::{self, Layout};

#[no_mangle]
unsafe extern "C" fn __abi_alloc(align: usize, new_len: usize) -> *mut u8 {
    if new_len == 0 {
        return align as *mut u8;
    }
    let layout = Layout::from_size_align_unchecked(new_len, align);
    let ptr = alloc::alloc(layout);
    if ptr.is_null() {
        alloc::handle_alloc_error(layout);
    }
    return ptr;
}

#[no_mangle]
unsafe extern "C" fn __abi_free(ptr: *mut u8, len: usize, align: usize) {
    if len == 0 {
        return;
    }
    let layout = Layout::from_size_align_unchecked(len, align);
    alloc::dealloc(ptr, layout);
}

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

struct PluginLogger;

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
        let record: Record = record.into();
        let data = rmp_serde::to_vec(&record).unwrap();
        unsafe { __log(data.len(), data.as_ptr()) }
    }

    fn flush(&self) {
        unsafe { __log_flush() }
    }
}

#[doc(hidden)]
pub unsafe fn __export(len: usize, data: *const u8, f: fn(Vec<RawValue>) -> RawValue) -> u64 {
    PluginLogger::init();
    let data = std::slice::from_raw_parts(data, len);
    let data = rmp_serde::from_slice(data).unwrap();
    let res = f(data);
    let data = rmp_serde::to_vec(&res).unwrap();
    let data = data.into_boxed_slice();
    let data = Box::leak(data);
    let len = data.len();
    let ptr = data.as_mut_ptr();
    ((len as u64) << 32) | (ptr as u64)
}

#[no_mangle]
unsafe extern "C" fn __export_free(len: usize, data: *mut u8) {
    let _data = Box::from_raw(std::slice::from_raw_parts_mut(data, len));
}

#[macro_export]
macro_rules! export {
    ($name:ident) => {
        $crate::concat_idents!(fn_name = __, $name {
            #[export_name = stringify!($name)]
            unsafe extern "C" fn fn_name(len: usize, data: *const u8) -> u64 {
                $crate::__export(len, data, $name)
            }
        });
    };
}
