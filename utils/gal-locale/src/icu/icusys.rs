#![allow(non_snake_case)]

pub type UErrorCode = i32;
pub const U_ZERO_ERROR: UErrorCode = 0i32;
pub const U_BUFFER_OVERFLOW_ERROR: UErrorCode = 15i32;

pub type UAcceptResult = i32;
pub const ULOC_ACCEPT_FAILED: UAcceptResult = 0i32;

pub type UChar = u16;

use std::ffi::c_char;

#[repr(C)]
pub struct UEnumeration(pub u8);

#[cfg_attr(target_os = "windows", link(name = "icu"))]
#[cfg_attr(target_os = "macos", link(name = "icucore"))]
extern "C" {
    pub fn uloc_getDefault() -> *const c_char;

    pub fn uloc_canonicalize(
        localeID: *const c_char,
        name: *mut c_char,
        nameCapacity: i32,
        err: *mut UErrorCode,
    ) -> i32;

    pub fn uloc_acceptLanguage(
        result: *mut c_char,
        resultAvailable: i32,
        outResult: *mut UAcceptResult,
        acceptList: *mut *const c_char,
        acceptListCount: i32,
        availableLocales: *mut UEnumeration,
        status: *mut UErrorCode,
    ) -> i32;

    pub fn uloc_getDisplayName(
        localeID: *const c_char,
        inLocaleID: *const c_char,
        result: *mut UChar,
        maxResultSize: i32,
        err: *mut UErrorCode,
    ) -> i32;

    pub fn uenum_openCharStringsEnumeration(
        strings: *const *const c_char,
        count: i32,
        ec: *mut UErrorCode,
    ) -> *mut UEnumeration;

    pub fn uenum_close(en: *mut UEnumeration);
}

/// The same macro as the one in `rust_icu_sys`.
#[macro_export]
macro_rules! versioned_function {
    ($func_name:path) => {
        $func_name
    };
}
