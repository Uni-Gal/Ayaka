cfg_if::cfg_if! {
    if #[cfg(any(target_os = "windows", target_os = "macos"))] {
        #[path = "icusys.rs"]
        mod platform;
    } else {
        #[path = "icu4c.rs"]
        mod platform;
    }
}

pub use platform::*;

use crate::*;
use thiserror::Error;

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

#[derive(Debug, Error)]
#[error("ICU error code: {0}")]
pub struct ICUError(UErrorCode);

pub type ICUResult<T> = std::result::Result<T, ICUError>;

unsafe fn call_with_buffer<T: UChar>(
    mut f: impl FnMut(*mut T, i32, *mut UErrorCode) -> i32,
) -> ICUResult<String> {
    let mut buffer = vec![T::default(); 10];
    let mut error_code = U_ZERO_ERROR;
    let mut len = f(buffer.as_mut_ptr(), buffer.len() as _, &mut error_code);
    if error_code == U_BUFFER_OVERFLOW_ERROR || len > buffer.len() as _ {
        buffer.resize(len as usize, T::default());
        len = f(buffer.as_mut_ptr(), buffer.len() as _, &mut error_code);
    }
    if error_code != U_ZERO_ERROR {
        return Err(ICUError(error_code));
    }
    if len > 0 {
        buffer.resize(len as usize, T::default());
    }
    Ok(T::string_from_buffer(buffer))
}

pub fn choose(
    accepts: impl IntoIterator<Item = impl AsRef<Locale>>,
    locales: impl IntoIterator<Item = impl AsRef<Locale>>,
) -> ICUResult<Option<LocaleBuf>> {
    let mut accepts_ptrs = accepts
        .into_iter()
        .map(|l| l.as_ref().0.as_ptr())
        .collect::<Vec<_>>();
    let locale_ptrs = locales
        .into_iter()
        .map(|l| l.as_ref().0.as_ptr())
        .collect::<Vec<_>>();
    let mut result = ULOC_ACCEPT_FAILED;
    let loc = unsafe {
        call_with_buffer::<u8>(|buffer, len, status| {
            let locales_enum = versioned_function!(uenum_openCharStringsEnumeration)(
                locale_ptrs.as_ptr() as _,
                locale_ptrs.len() as _,
                status,
            );
            if *status != U_ZERO_ERROR {
                return 0;
            }
            *status = U_ZERO_ERROR;
            let len = versioned_function!(uloc_acceptLanguage)(
                buffer as _,
                len,
                &mut result,
                accepts_ptrs.as_mut_ptr() as _,
                accepts_ptrs.len() as _,
                locales_enum,
                status,
            );
            versioned_function!(uenum_close)(locales_enum);
            len
        })
    }?;
    if result == ULOC_ACCEPT_FAILED {
        Ok(None)
    } else {
        Ok(Some(LocaleBuf(unsafe {
            CString::from_vec_unchecked(loc.into())
        })))
    }
}

pub fn current() -> &'static Locale {
    unsafe { Locale::new(CStr::from_ptr(versioned_function!(uloc_getDefault)() as _)) }
}

pub fn parse(s: &str) -> ICUResult<LocaleBuf> {
    let s = unsafe { CString::from_vec_unchecked(s.into()) };
    unsafe {
        call_with_buffer::<u8>(|buffer, len, status| {
            versioned_function!(uloc_canonicalize)(s.as_ptr() as _, buffer as _, len, status)
        })
    }
    .map(|s| LocaleBuf(unsafe { CString::from_vec_unchecked(s.into()) }))
}

pub fn native_name(loc: &Locale) -> ICUResult<String> {
    let loc_ptr = loc.0.as_ptr();
    unsafe {
        call_with_buffer::<u16>(|buffer, len, status| {
            versioned_function!(uloc_getDisplayName)(
                loc_ptr as _,
                loc_ptr as _,
                buffer,
                len,
                status,
            )
        })
    }
}
