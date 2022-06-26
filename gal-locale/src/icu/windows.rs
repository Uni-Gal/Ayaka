use super::*;
use windows_sys::Win32::Globalization::*;

pub use windows_sys::Win32::Globalization::{
    uenum_close, uenum_openCharStringsEnumeration, uloc_acceptLanguage, UAcceptResult, UErrorCode,
    ULOC_ACCEPT_FAILED, U_BUFFER_OVERFLOW_ERROR, U_ZERO_ERROR,
};

pub fn current() -> Option<Locale> {
    let lcid = unsafe { GetUserDefaultLCID() };
    unsafe {
        call_with_buffer(|buffer, len, status| uloc_getLocaleForLCID(lcid, buffer, len, status))
    }
    .map(Locale)
}
