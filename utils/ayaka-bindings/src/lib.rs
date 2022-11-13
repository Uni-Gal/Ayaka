#![feature(fn_traits)]
#![feature(tuple_trait)]
#![feature(unboxed_closures)]

#[doc(no_inline)]
pub use ayaka_bindings_types::*;
#[doc(no_inline)]
pub use ayaka_script::{RawValue, ValueType};
#[doc(no_inline)]
pub use log;

mod logger;

use scopeguard::defer;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    alloc::{self, Layout},
    marker::Tuple,
};

const ABI_ALIGN: usize = 8;

#[no_mangle]
unsafe extern "C" fn __abi_alloc(len: usize) -> *mut u8 {
    if len == 0 {
        return ABI_ALIGN as *mut u8;
    }
    let layout = Layout::from_size_align_unchecked(len, ABI_ALIGN);
    let ptr = alloc::alloc(layout);
    if ptr.is_null() {
        alloc::handle_alloc_error(layout);
    }
    return ptr;
}

#[no_mangle]
unsafe extern "C" fn __abi_free(ptr: *mut u8, len: usize) {
    if len == 0 {
        return;
    }
    let layout = Layout::from_size_align_unchecked(len, ABI_ALIGN);
    alloc::dealloc(ptr, layout);
}

unsafe fn __abi_alloc_from(data: &[u8]) -> (*mut u8, usize) {
    let ptr = __abi_alloc(data.len());
    let slice = std::slice::from_raw_parts_mut(ptr, data.len());
    slice.copy_from_slice(data);
    (slice.as_mut_ptr(), slice.len())
}

#[doc(hidden)]
pub unsafe fn __export<Params: DeserializeOwned + Tuple, Res: Serialize>(
    len: usize,
    data: *const u8,
    f: impl FnOnce<Params, Output = Res>,
) -> u64 {
    logger::PluginLogger::init();
    let data = std::slice::from_raw_parts(data, len);
    let data = rmp_serde::from_slice(data).unwrap();
    let res = f.call_once(data);
    let data = rmp_serde::to_vec(&res).unwrap();
    let (ptr, len) = __abi_alloc_from(&data);
    ((len as u64) << 32) | (ptr as u64)
}

#[doc(hidden)]
pub unsafe fn __import<Params: Serialize, Res: DeserializeOwned>(
    f: unsafe extern "C" fn(len: usize, data: *const u8) -> u64,
    args: Params,
) -> Res {
    let data = rmp_serde::to_vec(&args).unwrap();
    let res = f(data.len(), data.as_ptr());
    let (len, res) = ((res >> 32) as usize, (res & 0xFFFFFFFF) as *mut u8);
    defer! { __abi_free(res, len) };
    let data = std::slice::from_raw_parts_mut(res, len);
    let data = rmp_serde::from_slice(data).unwrap();
    data
}

pub use ayaka_bindings_impl::{export, import};
