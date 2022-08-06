//! The internal locale lib.
//!
//! This crate provides the [`Locale`] and [`LocaleBuf`] types.
//! They are internally a null-terminated string,
//! and use icu4c to parse and choose.

#![warn(missing_docs)]

mod icu;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::borrow::{Borrow, Cow};
use std::ffi::{CStr, CString};
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

/// Representation of a borrowed [`LocaleBuf`].
#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Locale(CStr);

impl Locale {
    pub(crate) unsafe fn new<'a>(loc: &'a CStr) -> &'a Self {
        &*(loc as *const CStr as *const Self)
    }

    /// Get the current locale of the system.
    /// Internally it calles `uloc_getDefault`.
    ///
    /// ```
    /// # use gal_locale::Locale;
    /// println!("Current locale: {}", Locale::current());
    /// ```
    pub fn current() -> &'static Self {
        icu::current()
    }

    /// Choose the best match from the provided locales.
    /// Internally it calls `uloc_acceptLanguage`.
    ///
    /// Returns [`None`] if it cannot choose a best match.
    ///
    /// ```
    /// # use gal_locale::LocaleBuf;
    /// let current = "zh_CN".parse::<LocaleBuf>().unwrap();
    /// let accepts = [
    ///     "en".parse::<LocaleBuf>().unwrap(),
    ///     "ja".parse().unwrap(),
    ///     "zh_Hans".parse().unwrap(),
    ///     "zh_Hant".parse().unwrap(),
    /// ];
    /// assert_eq!(
    ///     current.choose_from(&accepts).unwrap().unwrap().to_string(),
    ///     "zh_Hans"
    /// );
    /// ```
    pub fn choose_from(
        &self,
        locales: impl IntoIterator<Item = impl AsRef<Self>>,
    ) -> Result<Option<LocaleBuf>> {
        Ok(icu::choose([self], locales)?)
    }

    /// Get the native display name of the locale.
    /// Internally it calls `uloc_getDisplayName`.
    ///
    /// ```
    /// # use gal_locale::LocaleBuf;
    /// assert_eq!(
    ///     "en".parse::<LocaleBuf>().unwrap().native_name().unwrap(),
    ///     "English",
    /// );
    /// ```
    pub fn native_name(&self) -> Result<String> {
        Ok(icu::native_name(self)?)
    }
}

impl AsRef<Locale> for Locale {
    fn as_ref(&self) -> &Locale {
        self
    }
}

impl ToOwned for Locale {
    type Owned = LocaleBuf;

    fn to_owned(&self) -> Self::Owned {
        LocaleBuf(self.0.to_owned())
    }
}

impl<'a> From<&'a Locale> for Cow<'a, Locale> {
    fn from(loc: &'a Locale) -> Self {
        Cow::Borrowed(loc)
    }
}

impl Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.to_str().map_err(|_| std::fmt::Error)?)
    }
}

/// Represents an owned locale string.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(try_from = "String", into = "String")]
pub struct LocaleBuf(CString);

impl AsRef<Locale> for LocaleBuf {
    fn as_ref(&self) -> &Locale {
        unsafe { Locale::new(self.0.as_c_str()) }
    }
}

impl Deref for LocaleBuf {
    type Target = Locale;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Borrow<Locale> for LocaleBuf {
    fn borrow(&self) -> &Locale {
        self.as_ref()
    }
}

impl<'a> From<&'a LocaleBuf> for Cow<'a, Locale> {
    fn from(loc: &'a LocaleBuf) -> Self {
        Cow::Borrowed(loc)
    }
}

impl From<LocaleBuf> for Cow<'_, Locale> {
    fn from(loc: LocaleBuf) -> Self {
        Cow::Owned(loc)
    }
}

impl From<&LocaleBuf> for LocaleBuf {
    fn from(loc: &LocaleBuf) -> Self {
        loc.clone()
    }
}

impl From<&Locale> for LocaleBuf {
    fn from(loc: &Locale) -> Self {
        loc.to_owned()
    }
}

impl FromStr for LocaleBuf {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(icu::parse(s)?)
    }
}

impl TryFrom<String> for LocaleBuf {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(icu::parse(&value)?)
    }
}

impl Display for LocaleBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl From<LocaleBuf> for String {
    fn from(loc: LocaleBuf) -> Self {
        loc.0.to_str().map(|s| s.to_string()).unwrap_or_default()
    }
}

#[cfg(test)]
mod test {
    use crate::LocaleBuf;

    #[test]
    fn parse() {
        assert_eq!(
            "zh_Hans".parse::<LocaleBuf>().unwrap().to_string(),
            "zh_Hans"
        );
    }

    #[test]
    fn accept() {
        let current = "zh_CN".parse::<LocaleBuf>().unwrap();
        let accepts = [
            "en".parse::<LocaleBuf>().unwrap(),
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
