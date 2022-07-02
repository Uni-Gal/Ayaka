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

pub(crate) unsafe fn call_with_buffer(
    mut f: impl FnMut(*mut u8, i32, *mut UErrorCode) -> i32,
) -> ICUResult<CString> {
    let mut error_code = U_ZERO_ERROR;
    let len = f(null_mut(), 0, &mut error_code);
    if error_code > U_ZERO_ERROR && error_code != U_BUFFER_OVERFLOW_ERROR {
        return Err(ICUError::ICU(error_code));
    }
    error_code = U_ZERO_ERROR;
    let mut buffer = vec![0; len as usize + 1];
    f(buffer.as_mut_ptr(), len + 1, &mut error_code);
    if error_code > U_ZERO_ERROR {
        return Err(ICUError::ICU(error_code));
    }
    Ok(CString::from_vec_with_nul(buffer)?)
}

pub fn choose(
    accepts: impl Iterator<Item = impl Borrow<Locale>>,
    locales: impl Iterator<Item = impl Borrow<Locale>>,
) -> ICUResult<Option<Locale>> {
    let mut accepts_ptrs = accepts.map(|l| l.borrow().0.as_ptr()).collect::<Vec<_>>();
    let locale_ptrs = locales.map(|l| l.borrow().0.as_ptr()).collect::<Vec<_>>();
    let mut result = ULOC_ACCEPT_FAILED;
    let loc = unsafe {
        call_with_buffer(|buffer, len, status| {
            let locales_enum = imp_uenum_openCharStringsEnumeration(
                locale_ptrs.as_ptr(),
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
    Locale(unsafe { CStr::from_ptr(imp_uloc_getDefault() as _) }.to_owned())
}

pub fn parse(s: &str) -> ICUResult<Locale> {
    let s = CString::new(s)?;
    unsafe {
        call_with_buffer(|buffer, len, status| {
            imp_uloc_canonicalize(s.as_ptr() as _, buffer as _, len, status)
        })
    }
    .map(Locale)
}
