mod icu;

use serde::Deserialize;
use std::ffi::{CString, NulError};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Locale(CString);

impl FromStr for Locale {
    type Err = NulError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        CString::new(s).map(Self)
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
                v.parse().map_err(|e| Error::custom(e))
            }
        }
        deserializer.deserialize_any(LocaleVisitor)
    }
}

pub use icu::choose;

#[cfg(test)]
mod test {
    use crate::icu::*;

    #[test]
    fn accept() {
        let current = "zh_CN".parse().unwrap();
        let accepts = vec![
            "en".parse().unwrap(),
            "ja".parse().unwrap(),
            "zh_Hans".parse().unwrap(),
            "zh_Hant".parse().unwrap(),
        ];
        assert_eq!(choose_impl(current, accepts), "zh_Hans".parse().ok());
    }
}
