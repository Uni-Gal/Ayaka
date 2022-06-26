use super::*;
use widestring::U16CString;
use windows_sys::Win32::Globalization::*;

pub use windows_sys::Win32::Globalization::{
    uenum_close, uenum_openCharStringsEnumeration, uloc_acceptLanguage, uloc_forLanguageTag,
    UAcceptResult, UErrorCode, ULOC_ACCEPT_FAILED, U_BUFFER_OVERFLOW_ERROR, U_ZERO_ERROR,
};

pub fn current() -> Option<CString> {
    let locale_name = unsafe { U16CString::from_str_unchecked(LOCALE_NAME_SYSTEM_DEFAULT) };
    let len = unsafe { GetLocaleInfoEx(locale_name.as_ptr(), LOCALE_SNAME, null_mut(), 0) };
    if len > 0 {
        let mut buffer = vec![0; len as usize];
        unsafe { GetLocaleInfoEx(locale_name.as_ptr(), LOCALE_SNAME, buffer.as_mut_ptr(), len) };
        CString::new(unsafe { U16CString::from_vec_unchecked(buffer) }.to_string_lossy()).ok()
    } else {
        None
    }
}
