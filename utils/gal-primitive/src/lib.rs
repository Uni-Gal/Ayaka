//! The primitive types
//!
//! This crate provides the primitive type [`RawValue`].
//! It is used by scripts in `gal` project.
//! The value operation and type conversion are handled in `gal-runtime`.

#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// The basic and only type used in scripts.
/// ```
/// # use gal_primitive::RawValue;
/// assert_eq!(serde_yaml::from_str::<RawValue>("~").unwrap(), RawValue::Unit);
/// assert_eq!(serde_yaml::from_str::<RawValue>("true").unwrap(), RawValue::Bool(true));
/// assert_eq!(serde_yaml::from_str::<RawValue>("123").unwrap(), RawValue::Num(123));
/// assert_eq!(serde_yaml::from_str::<RawValue>("\"hello\"").unwrap(), RawValue::Str("hello".to_string()));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RawValue {
    /// The unit type. It is empty, just like [`None`] or [`()`] in Rust.
    Unit,
    /// The boolean type.
    Bool(bool),
    /// The number type. It's [`i64`].
    Num(i64),
    /// The string type.
    Str(String),
}

/// Represents the type of [`RawValue`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValueType {
    /// The unit type.
    Unit,
    /// The boolean type.
    Bool,
    /// The number type.
    Num,
    /// The string type.
    Str,
}

impl Default for RawValue {
    fn default() -> Self {
        Self::Unit
    }
}

impl RawValue {
    /// Gets [`ValueType`].
    pub fn get_type(&self) -> ValueType {
        match self {
            Self::Unit => ValueType::Unit,
            Self::Bool(_) => ValueType::Bool,
            Self::Num(_) => ValueType::Num,
            Self::Str(_) => ValueType::Str,
        }
    }

    /// Gets a boolean from the value:
    /// * A [`RawValue::Unit`] converts to `false`.
    /// * A [`RawValue::Num`] converts to `false` if and only if it's zero.
    /// * A [`RawValue::Str`] converts to `false` if and only if it's empty.
    ///
    /// ```
    /// # use gal_primitive::RawValue;
    /// let unit_value = RawValue::Unit;
    /// assert!(!unit_value.get_bool());
    /// let num_value = RawValue::Num(123);
    /// assert!(num_value.get_bool());
    /// let str_value = RawValue::Str("hello".to_string());
    /// assert!(str_value.get_bool());
    /// let empty_str_value = RawValue::Str(String::default());
    /// assert!(!empty_str_value.get_bool());
    /// ```
    pub fn get_bool(&self) -> bool {
        match self {
            Self::Unit => false,
            Self::Bool(b) => *b,
            Self::Num(i) => *i != 0,
            Self::Str(s) => !s.is_empty(),
        }
    }

    /// Gets a number from the value:
    /// * A [`RawValue::Unit`] converts to 0.
    /// * A [`RawValue::Bool`] converts `false` to 0 and `true` to 1.
    /// * A [`RawValue::Str`] converts to the length of the string.
    ///
    /// ```
    /// # use gal_primitive::RawValue;
    /// let unit_value = RawValue::Unit;
    /// assert_eq!(unit_value.get_num(), 0);
    /// let bool_value = RawValue::Bool(true);
    /// assert_eq!(bool_value.get_num(), 1);
    /// let str_value = RawValue::Str("hello".to_string());
    /// assert_eq!(str_value.get_num(), 5);
    /// ```
    pub fn get_num(&self) -> i64 {
        match self {
            Self::Unit => 0,
            Self::Bool(b) => *b as i64,
            Self::Num(i) => *i,
            Self::Str(s) => s.len() as i64,
        }
    }

    /// Gets a string from the value:
    /// * A [`RawValue::Unit`] converts to empty string.
    /// * A [`RawValue::Bool`] converts to "false" or "true".
    /// * A [`RawValue::Num`] converts to the string representation of the number.\
    ///
    /// Be careful to use `get_str().into_owned()`, if possible, use `into_str()` instead.
    ///
    /// ```
    /// # use gal_primitive::RawValue;
    /// let unit_value = RawValue::Unit;
    /// assert_eq!(unit_value.get_str(), "");
    /// let bool_value = RawValue::Bool(true);
    /// assert_eq!(bool_value.get_str(), "true");
    /// let num_value = RawValue::Num(123);
    /// assert_eq!(num_value.get_str(), "123");
    /// ```
    pub fn get_str(&self) -> Cow<str> {
        match self {
            Self::Unit => Cow::default(),
            Self::Bool(b) => b.to_string().into(),
            Self::Num(i) => i.to_string().into(),
            Self::Str(s) => s.as_str().into(),
        }
    }

    /// Gets a string from the value:
    /// * A [`RawValue::Unit`] converts to empty string.
    /// * A [`RawValue::Bool`] converts to "false" or "true".
    /// * A [`RawValue::Num`] converts to the string representation of the number.
    pub fn into_str(self) -> String {
        match self {
            Self::Unit => String::default(),
            Self::Bool(b) => b.to_string(),
            Self::Num(i) => i.to_string(),
            Self::Str(s) => s,
        }
    }
}

impl<'de> Deserialize<'de> for RawValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        struct ValueVisitor;

        impl<'de> serde::de::Visitor<'de> for ValueVisitor {
            type Value = RawValue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a unit, boolean, integer, string value")
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(RawValue::Unit)
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(RawValue::Bool(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(RawValue::Num(v))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(RawValue::Num(v as i64))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(RawValue::Str(v.into()))
            }
        }
        deserializer.deserialize_any(ValueVisitor)
    }
}

impl Serialize for RawValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Unit => serializer.serialize_unit(),
            Self::Bool(b) => serializer.serialize_bool(*b),
            Self::Num(n) => serializer.serialize_i64(*n),
            Self::Str(s) => serializer.serialize_str(s),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn serde_value() {
        assert_eq!(
            serde_yaml::from_str::<RawValue>("~").unwrap(),
            RawValue::Unit
        );

        assert_eq!(
            serde_yaml::from_str::<RawValue>("true").unwrap(),
            RawValue::Bool(true)
        );
        assert_eq!(
            serde_yaml::from_str::<RawValue>("false").unwrap(),
            RawValue::Bool(false)
        );

        assert_eq!(
            serde_yaml::from_str::<RawValue>("114514").unwrap(),
            RawValue::Num(114514)
        );
        assert_eq!(
            serde_yaml::from_str::<RawValue>("-1919810").unwrap(),
            RawValue::Num(-1919810)
        );

        assert_eq!(
            serde_yaml::from_str::<RawValue>("\"Hello world!\"").unwrap(),
            RawValue::Str("Hello world!".into())
        );

        assert_eq!(serde_yaml::to_string(&RawValue::Unit).unwrap(), "---\n~\n");

        assert_eq!(
            serde_yaml::to_string(&RawValue::Bool(true)).unwrap(),
            "---\ntrue\n"
        );
        assert_eq!(
            serde_yaml::to_string(&RawValue::Bool(false)).unwrap(),
            "---\nfalse\n"
        );

        assert_eq!(
            serde_yaml::to_string(&RawValue::Num(114514)).unwrap(),
            "---\n114514\n"
        );
        assert_eq!(
            serde_yaml::to_string(&RawValue::Num(-1919)).unwrap(),
            "---\n-1919\n"
        );

        assert_eq!(
            serde_yaml::to_string(&RawValue::Str("aaa".into())).unwrap(),
            "---\naaa\n"
        );
    }
}
