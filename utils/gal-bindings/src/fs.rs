// Only to work around WASI's complicated permissions.

use std::path::Path;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "fs")]
extern "C" {
    fn __exists(len: usize, data: *const u8) -> i32;
}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn __exists(len: usize, data: *const u8) -> i32 {
    let path = std::str::from_utf8_unchecked(std::slice::from_raw_parts(data, len));
    let path = std::path::PathBuf::from(path);
    path.exists() as i32
}

pub fn exists(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref().to_string_lossy();
    unsafe { __exists(path.len(), path.as_ptr()) != 0 }
}
