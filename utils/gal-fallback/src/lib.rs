//! A fallback type.
//!
//! [`Fallback`] type provides functionality to fallback
//! if a value or a part of value doesn't exist.

#![warn(missing_docs)]
#![deny(unsafe_code)]

/// Stores two [`Option`], and provides functionality to fallback.
///
/// Basically, you provides a function returns [`Option`],
/// and [`Fallback`] handles the fallback.
/// ```
/// # use gal_fallback::Fallback;
/// let data = Some("hello");
/// let base_data = Some("123");
/// let fallback = Fallback::new(data, base_data);
/// let num = fallback.and_then(|s| s.parse::<i32>().ok());
/// assert_eq!(num, Some(123));
/// ```
/// And you can map the [`Fallback`]:
/// ```
/// # use gal_fallback::Fallback;
/// let data = Some(123);
/// let base_data = Some(123456);
/// let fallback = Fallback::new(data, base_data);
/// let fallback = fallback.map(|i| i.to_string());
/// let s = fallback.and_then(|s| if s.len() > 3 { Some(s) } else { None });
/// assert_eq!(s, Some("123456".to_string()));
/// ```
pub struct Fallback<T> {
    data: Option<T>,
    base_data: Option<T>,
}

impl<T> Fallback<T> {
    /// Creates a new [`Fallback`].
    pub const fn new(data: Option<T>, base_data: Option<T>) -> Self {
        Self { data, base_data }
    }

    /// Returns `false` if both `data` and `base_data` are [`None`].
    pub const fn is_some(&self) -> bool {
        self.data.is_some() || self.base_data.is_some()
    }

    /// Converts from `&Fallback<T>` to `Fallback<&T>`.
    pub const fn as_ref(&self) -> Fallback<&T> {
        Fallback::new(self.data.as_ref(), self.base_data.as_ref())
    }

    /// Fallbacks the data or part of data.
    pub fn and_then<V>(self, mut f: impl FnMut(T) -> Option<V>) -> Option<V> {
        self.data
            .and_then(|t| f(t))
            .or_else(|| self.base_data.and_then(|t| f(t)))
    }

    /// Fallbacks the total data.
    pub fn fallback(self) -> Option<T> {
        self.data.or_else(|| self.base_data)
    }

    /// Maps to a new [`Fallback`].
    pub fn map<V>(self, mut f: impl FnMut(T) -> V) -> Fallback<V> {
        Fallback::new(self.data.map(|t| f(t)), self.base_data.map(|t| f(t)))
    }

    /// Exacts the `data` and `base_data`.
    pub fn unzip(self) -> (Option<T>, Option<T>) {
        (self.data, self.base_data)
    }
}

impl<T> Fallback<Option<T>> {
    /// Converts from `Fallback<Option<T>>` to `Fallback<T>`.
    pub fn flatten(self) -> Fallback<T> {
        Fallback::new(self.data.flatten(), self.base_data.flatten())
    }
}

#[doc(hidden)]
pub trait IsEmpty2 {
    fn is_empty2(&self) -> bool;
}

impl IsEmpty2 for String {
    fn is_empty2(&self) -> bool {
        self.is_empty()
    }
}

impl<T> IsEmpty2 for Vec<T> {
    fn is_empty2(&self) -> bool {
        self.is_empty()
    }
}

impl<K, V> IsEmpty2 for HashMap<K, V> {
    fn is_empty2(&self) -> bool {
        self.is_empty()
    }
}

impl<T: IsEmpty2> Fallback<T> {
    /// Treats the empty container as [`None`] and fallbacks.
    pub fn and_any(self) -> Option<T> {
        self.and_then(|s| if s.is_empty2() { None } else { Some(s) })
    }
}

impl<T> From<Fallback<T>> for Option<T> {
    fn from(f: Fallback<T>) -> Self {
        if f.data.is_some() {
            f.data
        } else {
            f.base_data
        }
    }
}

#[doc(hidden)]
pub struct FallbackIter<A> {
    data: A,
    base_data: A,
}

impl<A: Iterator> Iterator for FallbackIter<A> {
    type Item = Fallback<A::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let d = self.data.next();
        let based = self.base_data.next();
        if d.is_some() || based.is_some() {
            Some(Fallback::new(d, based))
        } else {
            None
        }
    }
}

impl<T> IntoIterator for Fallback<Vec<T>> {
    type Item = Fallback<T>;

    type IntoIter = FallbackIter<<Vec<T> as IntoIterator>::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        FallbackIter {
            data: self.data.unwrap_or_default().into_iter(),
            base_data: self.base_data.unwrap_or_default().into_iter(),
        }
    }
}

/// This trait helps to create a new fallback type.
///
/// The code
/// ```
/// # use gal_fallback::FallbackSpec;
/// #[derive(FallbackSpec)]
/// struct Foo {
///     data1: i32,
///     data2: String,
/// }
/// ```
/// is the same as
/// ```
/// # use gal_fallback::*;
/// struct Foo {
///     data1: i32,
///     data2: String,
/// }
///
/// struct FallbackFoo {
///     data1: Fallback<i32>,
///     data2: Fallback<String>,
/// }
///
/// impl FallbackSpec for Foo {
///     type SpecType = FallbackFoo;
/// }
///
/// impl From<Fallback<Foo>> for FallbackFoo {
///     fn from(data: Fallback<Foo>) -> Self {
///         let (data, base_data) = data.unzip();
///         let (data1, data2) = match data {
///             Some(data) => (Some(data.data1), Some(data.data2)),
///             None => (None, None),
///         };
///         let (base_data1, base_data2) = match base_data {
///             Some(data) => (Some(data.data1), Some(data.data2)),
///             None => (None, None),
///         };
///         Self {
///             data1: Fallback::new(data1, base_data1),
///             data2: Fallback::new(data2, base_data2),
///         }
///     }
/// }
/// ```
pub trait FallbackSpec: Sized {
    /// The specialized fallback type.
    type SpecType: From<Fallback<Self>>;
}

impl<T: FallbackSpec> Fallback<T> {
    /// Get the specialized fallback object.
    pub fn spec(self) -> T::SpecType {
        T::SpecType::from(self)
    }
}

use std::collections::HashMap;

pub use gal_fallback_derive::FallbackSpec;

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn some() {
        assert!(!Fallback::<()>::new(None, None).is_some());
    }

    #[test]
    fn option() {
        let f = Fallback::new(None, Some(100));
        assert_eq!(Option::from(f), Some(100));
    }
}
