//! The script parser.

use std::str::FromStr;

use crate::*;
use lalrpop_util::lalrpop_mod;
use serde::Deserialize;

/// A full script, a collection of expressions.
///
/// The last expression is the final value of the script.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub struct Program(pub Vec<Expr>);

/// An expression.
///
/// Two expressions should be splited with `;`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    /// A reference to a variable.
    Ref(Ref),
    /// A const value.
    Const(RawValue),
    /// A unary operation.
    Unary(UnaryOp, Box<Expr>),
    /// A binary operation.
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    /// A call to a function.
    Call(String, String, Vec<Expr>),
}

/// Unary operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// `+`
    Positive,
    /// `-`
    Negative,
    /// `!`
    Not,
}

/// Binary operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    /// Value operations.
    Val(ValBinaryOp),
    /// Logical operations.
    Logic(LogicBinaryOp),
    /// Assignment.
    Assign,
    /// Inplace value operations.
    Inplace(ValBinaryOp),
}

/// Value binary operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValBinaryOp {
    /// `+`
    Add,
    /// `-`
    Minus,
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `%`
    Mod,
    /// `&`
    And,
    /// `|`
    Or,
    /// `^`
    Xor,
}

/// Logical operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicBinaryOp {
    /// `&&`
    And,
    /// `||`
    Or,
    /// `==`
    Eq,
    /// `!=`
    Neq,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `>`
    Gt,
    /// `>=`
    Ge,
}

/// Reference of a variable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ref {
    /// A local variable.
    /// It is only accessible in current program.
    Var(String),
    /// A context variable.
    /// It is stored in the context.
    /// The variable name is prefixed with `$`.
    Ctx(String),
}

lalrpop_mod!(
    #[allow(missing_docs)]
    #[allow(dead_code)]
    #[allow(clippy::all)]
    grammer,
    "/exec/grammer.rs"
);

pub use grammer::{ConstParser, ExprParser, ProgramParser, RefParser};

impl FromStr for Program {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ProgramParser::new()
            .parse(s)
            .map_err(|e| anyhow::anyhow!("{}", e))
    }
}

impl TryFrom<String> for Program {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[cfg(test)]
mod test {
    use crate::exec::*;

    fn var(s: &str) -> Expr {
        Expr::Ref(Ref::Var(s.into()))
    }

    #[test]
    fn program() {
        assert_eq!(
            ProgramParser::new()
                .parse(
                    "foo(a);
                    foo.bar(a, b)"
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
            ExprParser::new().parse("!(a && b || c)").unwrap(),
            Expr::Unary(
                UnaryOp::Not,
                Box::new(Expr::Binary(
                    Box::new(Expr::Binary(
                        Box::new(var("a")),
                        BinaryOp::Logic(LogicBinaryOp::And),
                        Box::new(var("b"))
                    )),
                    BinaryOp::Logic(LogicBinaryOp::Or),
                    Box::new(var("c"))
                ))
            )
        );
        assert_eq!(
            ExprParser::new().parse("foo(a)").unwrap(),
            Expr::Call(String::default(), "foo".into(), vec![var("a")])
        );
        assert_eq!(
            ExprParser::new().parse("foo.bar(a, b)").unwrap(),
            Expr::Call("foo".into(), "bar".into(), vec![var("a"), var("b")])
        );
        assert_eq!(
            ExprParser::new().parse("a + (b * (c & d))").unwrap(),
            Expr::Binary(
                Box::new(var("a")),
                BinaryOp::Val(ValBinaryOp::Add),
                Box::new(Expr::Binary(
                    Box::new(var("b")),
                    BinaryOp::Val(ValBinaryOp::Mul),
                    Box::new(Expr::Binary(
                        Box::new(var("c")),
                        BinaryOp::Val(ValBinaryOp::And),
                        Box::new(var("d"))
                    ))
                ))
            )
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
            RawValue::Num(114514.into())
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
    }
}
