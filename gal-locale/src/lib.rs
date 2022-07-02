mod icu;

use icu::*;
use serde::Deserialize;
use std::borrow::Borrow;
use std::ffi::{CString, FromVecWithNulError, NulError};
use std::fmt::Display;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ICUError {
    #[error("ICU error code: {0}")]
    ICU(UErrorCode),
    #[error(transparent)]
    Nul(#[from] NulError),
    #[error(transparent)]
    FromVecNul(#[from] FromVecWithNulError),
}

pub type ICUResult<T> = std::result::Result<T, ICUError>;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Locale(CString);

impl Locale {
    pub fn current() -> Self {
        current()
    }

    pub fn choose_from(
        &self,
        locales: impl Iterator<Item = impl Borrow<Self>>,
    ) -> ICUResult<Option<Self>> {
        choose([self].into_iter(), locales)
    }
}

impl FromStr for Locale {
    type Err = ICUError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse(s)
    }
}

impl Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.to_str().unwrap())
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

#[cfg(test)]
mod test {
    use crate::Locale;

    #[test]
    fn parse() {
        assert_eq!("zh_Hans".parse::<Locale>().unwrap().to_string(), "zh_Hans");
    }

    #[test]
    fn accept() {
        let current = "zh_CN".parse::<Locale>().unwrap();
        let accepts = [
            "en".parse().unwrap(),
            "ja".parse().unwrap(),
            "zh_Hans".parse().unwrap(),
            "zh_Hant".parse().unwrap(),
        ];
        assert_eq!(
            current
                .choose_from(accepts.iter())
                .unwrap()
                .unwrap()
                .to_string(),
            "zh_Hans"
        );
    }
}
