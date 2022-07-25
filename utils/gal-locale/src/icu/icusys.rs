#![allow(non_snake_case)]

pub type UErrorCode = i32;
pub const U_ZERO_ERROR: UErrorCode = 0i32;
pub const U_BUFFER_OVERFLOW_ERROR: UErrorCode = 15i32;

pub type UAcceptResult = i32;
pub const ULOC_ACCEPT_FAILED: UAcceptResult = 0i32;

pub type UChar = u16;

#[repr(C)]
pub struct UEnumeration(pub u8);

#[cfg_attr(target_os = "windows", link(name = "icu"))]
#[cfg_attr(target_os = "macos", link(name = "icucore"))]
extern "C" {
    #[link_name = "uloc_getDefault"]
    pub fn imp_uloc_getDefault() -> *const ::std::os::raw::c_char;

    #[link_name = "uloc_canonicalize"]
    pub fn imp_uloc_canonicalize(
        localeID: *const ::std::os::raw::c_char,
        name: *mut ::std::os::raw::c_char,
        nameCapacity: i32,
        err: *mut UErrorCode,
    ) -> i32;

    #[link_name = "uloc_acceptLanguage"]
    pub fn imp_uloc_acceptLanguage(
        result: *mut ::std::os::raw::c_char,
        resultAvailable: i32,
        outResult: *mut UAcceptResult,
        acceptList: *mut *const ::std::os::raw::c_char,
        acceptListCount: i32,
        availableLocales: *mut UEnumeration,
        status: *mut UErrorCode,
    ) -> i32;

    #[link_name = "uloc_getDisplayName"]
    pub fn imp_uloc_getDisplayName(
        localeID: *const ::std::os::raw::c_char,
        inLocaleID: *const ::std::os::raw::c_char,
        result: *mut UChar,
        maxResultSize: i32,
        err: *mut UErrorCode,
    ) -> i32;

    #[link_name = "uenum_openCharStringsEnumeration"]
    pub fn imp_uenum_openCharStringsEnumeration(
        strings: *const *const ::std::os::raw::c_char,
        count: i32,
        ec: *mut UErrorCode,
    ) -> *mut UEnumeration;

    #[link_name = "uenum_close"]
    pub fn imp_uenum_close(en: *mut UEnumeration);
}
