use gal_script::{gal::ProgramParser, Program};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Bool(bool),
    Num(i64),
    Str(String),
    Expr(Arc<Program>),
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
                formatter.write_str("a boolean, integer, string value, or a piece of code.")
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
                if v.starts_with('{') && v.ends_with('}') {
                    Ok(Value::Expr(Arc::new(
                        ProgramParser::new().parse(&v[1..v.len() - 1]).unwrap(),
                    )))
                } else {
                    Ok(Value::Str(v.into()))
                }
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
            Self::Bool(b) => serializer.serialize_bool(*b),
            Self::Num(n) => serializer.serialize_i64(*n),
            Self::Str(s) => serializer.serialize_str(s),
            Self::Expr(_) => unimplemented!(),
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
#[serde(tag = "t", content = "c")]
pub enum Action {
    Text(Value),
    Switch(Vec<SwitchItem>),
}

#[derive(Debug, Deserialize)]
pub struct SwitchItem {
    pub text: String,
    pub enabled: Value,
    pub action: Value,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct RawContext {
    pub cur_para: String,
    pub cur_act: usize,
}

pub type VarMap = HashMap<String, Value>;

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
