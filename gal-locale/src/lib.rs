mod info;

use rust_icu_uloc::ULoc;

pub fn choose(languages: impl IntoIterator<Item = impl Into<ULoc>>) -> Option<ULoc> {
    if let Some(current) = info::current() {
        rust_icu_uloc::accept_language(languages, [current])
            .map(|(res, _)| res)
            .ok()
            .flatten()
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn accept() {
        let accepts = vec![
            ULoc::for_language_tag("en").unwrap(),
            ULoc::for_language_tag("ja").unwrap(),
            ULoc::for_language_tag("zh-Hans").unwrap(),
            ULoc::for_language_tag("zh_Hant").unwrap(),
        ];
        assert_eq!(choose(accepts), ULoc::for_language_tag("zh-Hans").ok());
    }
}
