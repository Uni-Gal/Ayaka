cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        #[path = "windows.rs"]
        mod platform;
    } else {
        #[path = "env.rs"]
        mod platform;
    }
}

pub use platform::*;

use crate::*;
use std::{
    ffi::{CStr, CString},
    ptr::null_mut,
};

trait UChar: Sized + Default + Copy {
    fn string_from_buffer(buffer: Vec<Self>) -> String;
}

impl UChar for u8 {
    fn string_from_buffer(buffer: Vec<Self>) -> String {
        unsafe { String::from_utf8_unchecked(buffer) }
    }
}

impl UChar for u16 {
    fn string_from_buffer(buffer: Vec<Self>) -> String {
        String::from_utf16_lossy(&buffer)
    }
}

unsafe fn call_with_buffer<T: UChar>(
    mut f: impl FnMut(*mut T, i32, *mut UErrorCode) -> i32,
) -> Result<String> {
    let mut error_code = U_ZERO_ERROR;
    let len = f(null_mut(), 0, &mut error_code);
    if error_code > U_ZERO_ERROR && error_code != U_BUFFER_OVERFLOW_ERROR {
        return Err(ICUError(error_code).into());
    }
    error_code = U_ZERO_ERROR;
    let mut buffer = vec![T::default(); len as usize];
    f(buffer.as_mut_ptr(), len, &mut error_code);
    if error_code > U_ZERO_ERROR {
        return Err(ICUError(error_code).into());
    }
    Ok(T::string_from_buffer(buffer))
}

pub fn choose(
    accepts: impl Iterator<Item = impl Borrow<Locale>>,
    locales: impl Iterator<Item = impl Borrow<Locale>>,
) -> Result<Option<Locale>> {
    let mut accepts_ptrs = accepts.map(|l| l.borrow().0.as_ptr()).collect::<Vec<_>>();
    let locale_ptrs = locales.map(|l| l.borrow().0.as_ptr()).collect::<Vec<_>>();
    let mut result = ULOC_ACCEPT_FAILED;
    let loc = unsafe {
        call_with_buffer::<u8>(|buffer, len, status| {
            let locales_enum = imp_uenum_openCharStringsEnumeration(
                locale_ptrs.as_ptr() as _,
                locale_ptrs.len() as _,
                status,
            );
            let len = imp_uloc_acceptLanguage(
                buffer as _,
                len,
                &mut result,
                accepts_ptrs.as_mut_ptr() as _,
                accepts_ptrs.len() as _,
                locales_enum,
                status,
            );
            imp_uenum_close(locales_enum);
            len
        })
    }
    .map(Locale)?;
    if result == ULOC_ACCEPT_FAILED {
        Ok(None)
    } else {
        Ok(Some(loc))
    }
}

pub fn current() -> Locale {
    Locale(
        unsafe { CStr::from_ptr(imp_uloc_getDefault() as _) }
            .to_str()
            .unwrap()
            .to_string(),
    )
}

pub fn parse(s: &str) -> Result<Locale> {
    let s = CString::new(s)?;
    unsafe {
        call_with_buffer::<u8>(|buffer, len, status| {
            imp_uloc_canonicalize(s.as_ptr() as _, buffer as _, len, status)
        })
    }
    .map(Locale)
}

pub fn native_name(loc: &Locale) -> Result<String> {
    let loc_ptr = loc.0.as_ptr();
    unsafe {
        call_with_buffer::<u16>(|buffer, len, status| {
            imp_uloc_getDisplayName(loc_ptr as _, loc_ptr as _, buffer, len, status)
        })
    }
}
