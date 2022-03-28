use lalrpop_util::lalrpop_mod;
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
    Call(String, Vec<Expr>),
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
                        bar(a, b)
                    }"
                )
                .unwrap(),
            Program(vec![
                Expr::Call("foo".into(), vec![var("a")]),
                Expr::Call("bar".into(), vec![var("a"), var("b")])
            ])
        );
    }

    #[test]
    fn expr() {
        assert_eq!(ExprParser::new().parse("a").unwrap(), var("a"));
        assert_eq!(
            ExprParser::new().parse("foo(a)").unwrap(),
            Expr::Call("foo".into(), vec![var("a")])
        );
        assert_eq!(
            ExprParser::new().parse("foo(a, b)").unwrap(),
            Expr::Call("foo".into(), vec![var("a"), var("b")])
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
}
