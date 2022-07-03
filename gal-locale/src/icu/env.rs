#![allow(non_snake_case)]

use rust_icu_sys::*;

pub use rust_icu_sys::{
    UAcceptResult,
    UAcceptResult::ULOC_ACCEPT_FAILED,
    UErrorCode,
    UErrorCode::{U_BUFFER_OVERFLOW_ERROR, U_ZERO_ERROR},
};

pub unsafe fn imp_uloc_getDefault() -> *const ::std::os::raw::c_char {
    versioned_function!(uloc_getDefault)()
}

pub unsafe fn imp_uloc_canonicalize(
    localeID: *const ::std::os::raw::c_char,
    name: *mut ::std::os::raw::c_char,
    nameCapacity: i32,
    err: *mut UErrorCode,
) -> i32 {
    versioned_function!(uloc_canonicalize)(localeID, name, nameCapacity, err)
}

pub unsafe fn imp_uloc_acceptLanguage(
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

pub fn imp_uloc_getDisplayName(
    localeID: *const ::std::os::raw::c_char,
    inLocaleID: *const ::std::os::raw::c_char,
    result: *mut UChar,
    maxResultSize: i32,
    err: *mut UErrorCode,
) -> i32 {
    versioned_function!(uloc_getDisplayName)(localeID, inLocaleID, result, maxResultSize, err)
}

pub unsafe fn imp_uenum_openCharStringsEnumeration(
    strings: *const *const ::std::os::raw::c_char,
    count: i32,
    ec: *mut UErrorCode,
) -> *mut UEnumeration {
    versioned_function!(uenum_openCharStringsEnumeration)(strings, count, ec)
}

pub unsafe fn imp_uenum_close(en: *mut UEnumeration) {
    versioned_function!(uenum_close)(en)
}
