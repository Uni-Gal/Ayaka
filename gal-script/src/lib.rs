use fp_bindgen::prelude::*;
use lalrpop_util::lalrpop_mod;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

lalrpop_mod!(gal);

pub use gal::*;

#[derive(Debug, PartialEq, Eq)]
pub struct Program(pub Vec<Expr>);

#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    Ref(Ref),
    Const(RawValue),
    Unary(UnaryOp, Box<Expr>),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    Call(String, String, Vec<Expr>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Positive,
    Negative,
    Not,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Val(ValBinaryOp),
    Logic(LogicBinaryOp),
    Assign,
    Inplace(ValBinaryOp),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ValBinaryOp {
    Add,
    Minus,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Xor,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LogicBinaryOp {
    And,
    Or,
    Eq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Ref {
    Var(String),
    Ctx(String),
    Res(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serializable)]
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
                formatter.write_str("a boolean, integer, string value, or a piece of code")
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

    fn var(s: &str) -> Expr {
        Expr::Ref(Ref::Var(s.into()))
    }

    #[test]
    fn program() {
        assert_eq!(
            ProgramParser::new()
                .parse(
                    "{
                        foo(a);
                        foo.bar(a, b)
                    }"
                )
                .unwrap(),
            Program(vec![
                Expr::Call(String::default(), "foo".into(), vec![var("a")]),
                Expr::Call("foo".into(), "bar".into(), vec![var("a"), var("b")])
            ])
        );
    }

    #[test]
    fn expr() {
        assert_eq!(ExprParser::new().parse("a").unwrap(), var("a"));
        assert_eq!(
            ExprParser::new().parse("foo(a)").unwrap(),
            Expr::Call(String::default(), "foo".into(), vec![var("a")])
        );
        assert_eq!(
            ExprParser::new().parse("foo.bar(a, b)").unwrap(),
            Expr::Call("foo".into(), "bar".into(), vec![var("a"), var("b")])
        );
    }

    #[test]
    fn r#const() {
        assert_eq!(ConstParser::new().parse("~").unwrap(), RawValue::Unit);

        assert_eq!(
            ConstParser::new().parse("true").unwrap(),
            RawValue::Bool(true)
        );
        assert_eq!(
            ConstParser::new().parse("false").unwrap(),
            RawValue::Bool(false)
        );

        assert_eq!(
            ConstParser::new().parse("114514").unwrap(),
            RawValue::Num(114514)
        );

        assert_eq!(
            ConstParser::new().parse("\"Hello world!\"").unwrap(),
            RawValue::Str("Hello world!".into())
        );
    }

    #[test]
    fn r#ref() {
        assert_eq!(RefParser::new().parse("a").unwrap(), Ref::Var("a".into()));
        assert_eq!(RefParser::new().parse("$a").unwrap(), Ref::Ctx("a".into()));
        assert_eq!(RefParser::new().parse("#a").unwrap(), Ref::Res("a".into()));
    }

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
