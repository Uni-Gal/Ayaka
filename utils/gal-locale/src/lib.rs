//! The internal locale lib.
//!
//! This crate provides the [`Locale`] type.

#![warn(missing_docs)]
#![deny(unsafe_code)]
#![feature(once_cell)]

mod matcher;

use icu_locid::LanguageIdentifier;
use matcher::LanguageMatcher;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr, sync::LazyLock};
use sys_locale::get_locale;
use thiserror::Error;

static MATCHER: LazyLock<LanguageMatcher> = LazyLock::new(|| LanguageMatcher::new());

/// Representation of a borrowed [`Locale`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Locale(pub LanguageIdentifier);

impl Locale {
    /// Get the current locale of the system.
    /// Internally it calles `uloc_getDefault`.
    ///
    /// ```
    /// # use gal_locale::Locale;
    /// println!("Current locale: {}", Locale::current());
    /// ```
    pub fn current() -> Self {
        get_locale()
            .and_then(|loc| loc.parse().ok())
            .unwrap_or_else(|| locale!("en"))
    }

    /// Choose the best match from the provided locales.
    /// Internally it calls `uloc_acceptLanguage`.
    ///
    /// Returns [`None`] if it cannot choose a best match.
    ///
    /// ```
    /// # use gal_locale::Locale;
    /// let current = "zh-CN".parse::<Locale>().unwrap();
    /// let accepts = [
    ///     "en".parse::<Locale>().unwrap(),
    ///     "ja".parse().unwrap(),
    ///     "zh-Hans".parse().unwrap(),
    ///     "zh-Hant".parse().unwrap(),
    /// ];
    /// assert_eq!(
    ///     current.choose_from(accepts).unwrap().to_string(),
    ///     "zh-Hans"
    /// );
    /// ```
    pub fn choose_from(&self, locales: impl IntoIterator<Item = Self>) -> Option<Self> {
        MATCHER
            .matches(self.0.clone(), locales.into_iter().map(|loc| loc.0))
            .map(|(lang, _)| Self(lang))
    }
}

impl Default for Locale {
    fn default() -> Self {
        locale!("en")
    }
}

impl Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for Locale {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

#[derive(Debug, Error)]
#[error("{0}")]
#[doc(hidden)]
pub struct ParserError(icu_locid::ParserError);

impl From<icu_locid::ParserError> for ParserError {
    fn from(err: icu_locid::ParserError) -> Self {
        Self(err)
    }
}

#[doc(hidden)]
pub use icu_locid::langid;

/// A macro allowing for compile-time construction of valid [`Locale`].
/// See [`icu_locid::langid!`].
///
/// ```
/// # use gal_locale::{locale, Locale};
/// const ZH_CN: Locale = locale!("zh_CN");
/// let zh_cn: Locale = "zh_CN".parse().unwrap();
/// assert_eq!(ZH_CN, zh_cn);
/// ```
#[macro_export]
macro_rules! locale {
    ($langid:literal) => {
        $crate::Locale($crate::langid!($langid))
    };
}

#[cfg(test)]
mod test {
    use crate::locale;

    #[test]
    fn parse() {
        assert_eq!(locale!("zh-Hans").to_string(), "zh-Hans");
    }

    #[test]
    fn accept() {
        let accepts = [
            locale!("en"),
            locale!("ja"),
            locale!("zh-Hans"),
            locale!("zh-Hant"),
        ];
        assert_eq!(
            locale!("zh-CN").choose_from(accepts.clone()),
            Some(locale!("zh-Hans"))
        );
        assert_eq!(
            locale!("zh-TW").choose_from(accepts.clone()),
            Some(locale!("zh-Hant"))
        );
    }
}
