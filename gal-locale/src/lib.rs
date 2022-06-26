mod info;

use rust_icu_uloc::ULoc;
use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Locale(ULoc);

impl FromStr for Locale {
    type Err = rust_icu_common::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ULoc::for_language_tag(s).map(Self)
    }
}

impl<'de> Deserialize<'de> for Locale {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        struct LocaleVisitor;

        impl<'de> serde::de::Visitor<'de> for LocaleVisitor {
            type Value = Locale;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a locale value")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                v.parse().map_err(|e| serde::de::Error::custom(e))
            }
        }
        deserializer.deserialize_any(LocaleVisitor)
    }
}

pub fn choose<'a>(languages: impl IntoIterator<Item = Locale>) -> Option<Locale> {
    if let Some(current) = info::current() {
        rust_icu_uloc::accept_language(languages.into_iter().map(|locale| locale.0), [current])
            .map(|(res, _)| res)
            .ok()
            .flatten()
            .map(Locale)
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
            "en".parse().unwrap(),
            "ja".parse().unwrap(),
            "zh-Hans".parse().unwrap(),
            "zh-Hant".parse().unwrap(),
        ];
        assert_eq!(choose(accepts), "zh-Hans".parse().ok());
    }
}
