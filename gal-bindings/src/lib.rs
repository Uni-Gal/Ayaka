pub use concat_idents::concat_idents;
pub use gal_primitive::*;
pub use log;

use log::Log;

fn forget_string(s: String) -> (i32, i32) {
    let v = s.into_bytes().into_boxed_slice();
    let len = v.len() as i32;
    let data = v.as_ptr() as i32;
    std::mem::forget(v);
    (len, data)
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "log")]
extern "C" {
    fn __log(level: i32, target_len: i32, target: i32, msg_len: i32, msg: i32);
    fn __log_flush();
}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn __log(_level: i32, _target_len: i32, _target: i32, _msg_len: i32, _msg: i32) {}

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
        let (target_len, target) = forget_string(record.target().to_string());
        let (msg_len, msg) = forget_string(format!("{}", record.args()));
        unsafe { __log(record.level() as i32, target_len, target, msg_len, msg) }
    }

    fn flush(&self) {
        unsafe { __log_flush() }
    }
}

pub unsafe fn __export(arg0: i32, arg1: i32, f: fn(Vec<RawValue>) -> RawValue) -> i32 {
    let base1 = arg0;
    let len1 = arg1;
    let mut result1 = Vec::with_capacity(len1 as usize);
    for i in 0..len1 {
        let base = base1 + i * 16;
        result1.push(match i32::from(*((base + 0) as *const u8)) {
            0 => RawValue::Unit,
            1 => RawValue::Bool(match i32::from(*((base + 8) as *const u8)) {
                0 => false,
                _ => true,
            }),
            2 => RawValue::Num(*((base + 8) as *const i64)),
            _ => RawValue::Str({
                let len0 = *((base + 12) as *const i32) as usize;

                String::from_utf8_unchecked(Vec::from_raw_parts(
                    *((base + 8) as *const i32) as *mut _,
                    len0,
                    len0,
                ))
            }),
        });
    }
    std::alloc::dealloc(
        base1 as *mut _,
        std::alloc::Layout::from_size_align_unchecked((len1 as usize) * 16, 8),
    );
    let result2 = f(result1);
    let (result5_0, result5_1, result5_2) = match result2 {
        RawValue::Unit => (0i32, 0i64, 0i32),
        RawValue::Bool(e) => {
            let result3 = match e {
                false => 0i32,
                true => 1i32,
            };

            (1i32, i64::from(result3), 0i32)
        }
        RawValue::Num(e) => (2i32, wit_bindgen_rust::rt::as_i64(e), 0i32),
        RawValue::Str(e) => {
            let vec4 = (e.into_bytes()).into_boxed_slice();
            let ptr4 = vec4.as_ptr() as i32;
            let len4 = vec4.len() as i32;
            core::mem::forget(vec4);

            (3i32, i64::from(ptr4), len4)
        }
    };
    let ptr6 = RET_AREA.as_mut_ptr() as i32;
    *((ptr6 + 16) as *mut i32) = result5_2;
    *((ptr6 + 8) as *mut i64) = result5_1;
    *((ptr6 + 0) as *mut i32) = result5_0;
    ptr6
}

static mut RET_AREA: [i64; 4] = [0; 4];

#[macro_export]
macro_rules! export {
    ($name:ident) => {
        $crate::concat_idents!(fn_name = __, $name {
            #[export_name = stringify!($name)]
            unsafe extern "C" fn fn_name(arg0: i32, arg1: i32) -> i32 {
                $crate::__export(arg0, arg1, $name)
            }
        });
    };
}
