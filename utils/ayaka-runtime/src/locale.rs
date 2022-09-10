use icu_locid::{LanguageIdentifier, ParserError};
use language_matcher::LanguageMatcher;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr, sync::LazyLock};
use sys_locale::get_locale;

static MATCHER: LazyLock<LanguageMatcher> = LazyLock::new(LanguageMatcher::new);

/// Representation of a language identifier.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Locale(pub LanguageIdentifier);

impl Locale {
    /// Get the current locale of the system.
    /// Internally it calles [`sys_locale::get_locale`].
    ///
    /// ```
    /// # use ayaka_runtime::Locale;
    /// println!("Current locale: {}", Locale::current());
    /// ```
    pub fn current() -> Self {
        get_locale()
            .and_then(|loc| loc.parse().ok())
            .unwrap_or_default()
    }

    /// Choose the best match from the provided locales.
    ///
    /// Returns [`None`] if it cannot choose a best match.
    ///
    /// ```
    /// # use ayaka_runtime::locale;
    /// let current = locale!("zh-CN");
    /// let accepts = [
    ///     locale!("en"),
    ///     locale!("ja"),
    ///     locale!("zh-Hans"),
    ///     locale!("zh-Hant"),
    /// ];
    /// assert_eq!(
    ///     current.choose_from(&accepts),
    ///     Some(&locale!("zh-Hans")),
    /// );
    /// ```
    pub fn choose_from<'a>(
        &self,
        locales: impl IntoIterator<Item = impl Into<&'a Locale>>,
    ) -> Option<&'a Locale> {
        MATCHER
            .matches(self.0.clone(), locales.into_iter().map(|loc| loc.into()))
            .map(|(lang, _)| lang)
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

impl AsRef<LanguageIdentifier> for Locale {
    fn as_ref(&self) -> &LanguageIdentifier {
        &self.0
    }
}

#[doc(hidden)]
pub use icu_locid::langid as __icu_langid;

/// A macro allowing for compile-time construction of valid [`Locale`].
/// See [`icu_locid::langid!`].
///
/// ```
/// # use ayaka_runtime::{locale, Locale};
/// const ZH_CN: Locale = locale!("zh_CN");
/// let zh_cn: Locale = "zh_CN".parse().unwrap();
/// assert_eq!(ZH_CN, zh_cn);
/// ```
#[macro_export]
macro_rules! locale {
    ($langid:literal) => {
        $crate::Locale($crate::__icu_langid!($langid))
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
            locale!("zh-CN").choose_from(&accepts),
            Some(&locale!("zh-Hans"))
        );
        assert_eq!(
            locale!("zh-TW").choose_from(&accepts),
            Some(&locale!("zh-Hant"))
        );
    }
}
