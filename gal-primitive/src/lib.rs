use gal_script::{gal::ProgramParser, Program};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Unit,
    Bool(bool),
    Num(i64),
    Str(String),
    Expr(Arc<Program>),
}

impl Default for Value {
    fn default() -> Self {
        Self::Unit
    }
}

impl From<RawValue> for Value {
    fn from(v: RawValue) -> Self {
        match v {
            RawValue::Unit => Self::Unit,
            RawValue::Bool(b) => Self::Bool(b),
            RawValue::Num(i) => Self::Num(i),
            RawValue::Str(s) => Self::Str(s),
        }
    }
}

impl Value {
    pub(crate) fn bool_true() -> Self {
        Self::Bool(true)
    }

    pub(crate) fn from_str(s: &str) -> Self {
        match ProgramParser::new().parse(s) {
            Ok(p) => Self::Expr(Arc::new(p)),
            Err(_) => Self::Str(s.into()),
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        struct ValueVisitor;

        impl<'de> serde::de::Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a boolean, integer, string value, or a piece of code")
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Value::Unit)
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Value::Bool(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Value::Num(v))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Value::Num(v as i64))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Value::from_str(v))
            }
        }
        deserializer.deserialize_any(ValueVisitor)
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Unit => serializer.serialize_unit(),
            Self::Bool(b) => serializer.serialize_bool(*b),
            Self::Num(n) => serializer.serialize_i64(*n),
            Self::Str(s) => serializer.serialize_str(s),
            Self::Expr(_) => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RawValue {
    Unit,
    Bool(bool),
    Num(i64),
    Str(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValueType {
    Unit,
    Bool,
    Num,
    Str,
}

impl Default for RawValue {
    fn default() -> Self {
        Self::Unit
    }
}

impl RawValue {
    pub fn get_type(&self) -> ValueType {
        match self {
            Self::Unit => ValueType::Unit,
            Self::Bool(_) => ValueType::Bool,
            Self::Num(_) => ValueType::Num,
            Self::Str(_) => ValueType::Str,
        }
    }

    pub fn get_bool(&self) -> bool {
        match self {
            Self::Unit => false,
            Self::Bool(b) => *b,
            Self::Num(i) => *i != 0,
            Self::Str(s) => !s.is_empty(),
        }
    }

    pub fn get_num(&self) -> i64 {
        match self {
            Self::Unit => 0,
            Self::Bool(b) => *b as i64,
            Self::Num(i) => *i,
            Self::Str(s) => s.len() as i64,
        }
    }

    pub fn get_str(&self) -> Cow<str> {
        match self {
            Self::Unit => Cow::default(),
            Self::Bool(b) => b.to_string().into(),
            Self::Num(i) => i.to_string().into(),
            Self::Str(s) => s.as_str().into(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Game {
    pub title: String,
    pub author: String,
    pub paras: Vec<Paragraph>,
}

impl Game {
    pub fn find_para(&self, tag: &str) -> Option<&Paragraph> {
        for p in &self.paras {
            if p.tag == tag {
                return Some(p);
            }
        }
        None
    }
}

#[derive(Debug, Deserialize)]
pub struct Paragraph {
    pub tag: String,
    pub title: String,
    pub actions: Vec<Action>,
    pub next: Value,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Action {
    Text(Value),
    Switch {
        #[serde(default)]
        bind: String,
        #[serde(default)]
        allow_default: bool,
        items: Vec<SwitchItem>,
    },
}

#[derive(Debug, Deserialize)]
pub struct SwitchItem {
    pub text: String,
    #[serde(default = "Value::bool_true")]
    pub enabled: Value,
    #[serde(default)]
    pub action: Value,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct RawContext {
    pub cur_para: String,
    pub cur_act: usize,
}

pub type VarMap = HashMap<String, RawValue>;

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn serde_value() {
        assert_eq!(
            serde_yaml::from_str::<Value>("123").unwrap(),
            Value::Num(123)
        )
    }
}
