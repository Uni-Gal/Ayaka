mod icu;

use anyhow::Result;
use icu::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("ICU error code: {0}")]
pub struct ICUError(UErrorCode);

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Locale(String);

impl Locale {
    pub fn current() -> Self {
        current()
    }

    pub fn choose_from(
        &self,
        locales: impl IntoIterator<Item = impl AsRef<Self>>,
    ) -> Result<Option<Self>> {
        choose([self], locales)
    }

    pub fn native_name(&self) -> Result<String> {
        native_name(self)
    }
}

impl AsRef<Locale> for &'_ Locale {
    fn as_ref(&self) -> &Locale {
        self
    }
}

impl FromStr for Locale {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse(s)
    }
}

impl TryFrom<String> for Locale {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<Locale> for String {
    fn from(val: Locale) -> Self {
        val.to_string()
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
            "en".parse::<Locale>().unwrap(),
            "ja".parse().unwrap(),
            "zh_Hans".parse().unwrap(),
            "zh_Hant".parse().unwrap(),
        ];
        assert_eq!(
            current.choose_from(&accepts).unwrap().unwrap().to_string(),
            "zh_Hans"
        );
    }
}
