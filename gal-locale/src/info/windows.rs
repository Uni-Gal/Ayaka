use rust_icu_uloc::ULoc;
use windows::Win32::Globalization::*;

pub fn current() -> Option<ULoc> {
    const LOCALE_NAME_MAX_LENGTH: usize = 85;
    let mut buffer = [0; LOCALE_NAME_MAX_LENGTH];
    let len = unsafe { GetUserDefaultLocaleName(&mut buffer) };
    if len > 0 {
        let name = String::from_utf16_lossy(&buffer[..(len as usize - 1)]);
        ULoc::for_language_tag(&name).ok()
    } else {
        None
    }
}
