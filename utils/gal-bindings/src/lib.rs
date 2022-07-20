#![feature(fn_traits)]
#![feature(unboxed_closures)]

pub use concat_idents::concat_idents;
pub use gal_bindings_types::*;
pub use gal_primitive::*;
pub use log;

pub mod fs;
mod logger;

use serde::{de::DeserializeOwned, Serialize};
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

#[doc(hidden)]
pub unsafe fn __export<Params: DeserializeOwned, Res: Serialize>(
    len: usize,
    data: *const u8,
    f: impl FnOnce<Params, Output = Res>,
) -> u64 {
    logger::PluginLogger::init();
    let data = std::slice::from_raw_parts(data, len);
    let data = rmp_serde::from_slice(data).unwrap();
    let res = f.call_once(data);
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

pub use gal_bindings_impl::export;
