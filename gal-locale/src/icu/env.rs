#![allow(non_snake_case)]

use super::*;
use rust_icu_sys::*;

pub use rust_icu_sys::{
    UAcceptResult,
    UAcceptResult::ULOC_ACCEPT_FAILED,
    UErrorCode,
    UErrorCode::{U_BUFFER_OVERFLOW_ERROR, U_ZERO_ERROR},
};

pub unsafe fn uloc_acceptLanguage(
    result: *mut ::std::os::raw::c_char,
    resultAvailable: i32,
    outResult: *mut UAcceptResult,
    acceptList: *mut *const ::std::os::raw::c_char,
    acceptListCount: i32,
    availableLocales: *mut UEnumeration,
    status: *mut UErrorCode,
) -> i32 {
    versioned_function!(uloc_acceptLanguage)(
        result,
        resultAvailable,
        outResult,
        acceptList,
        acceptListCount,
        availableLocales,
        status,
    )
}

pub unsafe fn uenum_openCharStringsEnumeration(
    strings: *const *const ::std::os::raw::c_char,
    count: i32,
    ec: *mut UErrorCode,
) -> *mut UEnumeration {
    versioned_function!(uenum_openCharStringsEnumeration)(strings, count, ec)
}

pub unsafe fn uenum_close(en: *mut UEnumeration) {
    versioned_function!(uenum_close)(en)
}

pub fn current() -> Option<Locale> {
    std::env::var("LANG")
        .ok()
        .and_then(|lang| {
            let lang = CString::new(lang).unwrap();
            unsafe {
                call_with_buffer(|buffer, len, status| {
                    versioned_function!(uloc_forLanguageTag)(
                        lang.as_ptr(),
                        buffer as _,
                        len,
                        null_mut(),
                        status,
                    )
                })
            }
        })
        .map(Locale)
}