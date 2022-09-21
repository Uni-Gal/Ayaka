use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// The basic and only type used in scripts.
/// ```
/// # use ayaka_script::RawValue;
/// assert_eq!(serde_yaml::from_str::<RawValue>("~").unwrap(), RawValue::Unit);
/// assert_eq!(serde_yaml::from_str::<RawValue>("true").unwrap(), RawValue::Bool(true));
/// assert_eq!(serde_yaml::from_str::<RawValue>("123").unwrap(), RawValue::Num(123));
/// assert_eq!(serde_yaml::from_str::<RawValue>("\"hello\"").unwrap(), RawValue::Str("hello".to_string()));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(untagged)]
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
    /// # use ayaka_script::RawValue;
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
    /// # use ayaka_script::RawValue;
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
    /// # use ayaka_script::RawValue;
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

#[cfg(feature = "rt-format")]
use rt_format::{Format, FormatArgument, Specifier};

#[cfg(feature = "rt-format")]
impl FormatArgument for RawValue {
    fn supports_format(&self, specifier: &Specifier) -> bool {
        match self {
            RawValue::Unit | RawValue::Bool(_) | RawValue::Str(_) => {
                matches!(specifier.format, Format::Debug | Format::Display)
            }
            RawValue::Num(_) => true,
        }
    }

    fn fmt_display(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::fmt::Display;
        match self {
            RawValue::Unit => Ok(()),
            RawValue::Bool(b) => b.fmt(f),
            RawValue::Num(n) => n.fmt(f),
            RawValue::Str(s) => s.fmt(f),
        }
    }

    fn fmt_debug(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::fmt::Debug;
        self.fmt(f)
    }

    fn fmt_octal(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RawValue::Num(n) => std::fmt::Octal::fmt(n, f),
            _ => Err(std::fmt::Error),
        }
    }

    fn fmt_lower_hex(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RawValue::Num(n) => std::fmt::LowerHex::fmt(n, f),
            _ => Err(std::fmt::Error),
        }
    }

    fn fmt_upper_hex(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RawValue::Num(n) => std::fmt::UpperHex::fmt(n, f),
            _ => Err(std::fmt::Error),
        }
    }

    fn fmt_binary(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RawValue::Num(n) => std::fmt::Binary::fmt(n, f),
            _ => Err(std::fmt::Error),
        }
    }

    fn fmt_lower_exp(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RawValue::Num(n) => std::fmt::LowerExp::fmt(n, f),
            _ => Err(std::fmt::Error),
        }
    }

    fn fmt_upper_exp(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RawValue::Num(n) => std::fmt::UpperExp::fmt(n, f),
            _ => Err(std::fmt::Error),
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

        assert_eq!(serde_yaml::to_string(&RawValue::Unit).unwrap(), "null\n");

        assert_eq!(
            serde_yaml::to_string(&RawValue::Bool(true)).unwrap(),
            "true\n"
        );
        assert_eq!(
            serde_yaml::to_string(&RawValue::Bool(false)).unwrap(),
            "false\n"
        );

        assert_eq!(
            serde_yaml::to_string(&RawValue::Num(114514)).unwrap(),
            "114514\n"
        );
        assert_eq!(
            serde_yaml::to_string(&RawValue::Num(-1919)).unwrap(),
            "-1919\n"
        );

        assert_eq!(
            serde_yaml::to_string(&RawValue::Str("aaa".into())).unwrap(),
            "aaa\n"
        );
    }
}
