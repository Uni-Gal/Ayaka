mod icu;

use anyhow::Result;
use icu::*;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::fmt::Display;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("ICU error code: {0}")]
pub struct ICUError(UErrorCode);

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Locale(String);

impl Locale {
    pub fn current() -> Self {
        current()
    }

    pub fn choose_from(
        &self,
        locales: impl Iterator<Item = impl Borrow<Self>>,
    ) -> Result<Option<Self>> {
        choose([self].into_iter(), locales)
    }

    pub fn native_name(&self) -> Result<String> {
        native_name(self)
    }
}

impl FromStr for Locale {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse(s)
    }
}

impl Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
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

impl Serialize for Locale {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
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
