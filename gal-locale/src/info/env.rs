use rust_icu_uloc::ULoc;

pub fn current() -> Option<ULoc> {
    std::env::var("LANG")
        .ok()
        .and_then(|lang| ULoc::for_language_tag(&lang).ok())
}
