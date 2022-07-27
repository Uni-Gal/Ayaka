//! The script parser.

use crate::*;
use lalrpop_util::lalrpop_mod;
use serde::{Deserialize, Serialize};

lalrpop_mod!(grammer, "/exec/grammer.rs");

pub use grammer::*;

/// A full script, a collection of expressions.
///
/// The last expression is the final value of the script.
#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Program(pub Vec<Expr>);

/// An expression.
///
/// Two expressions should be splited with `;`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum UnaryOp {
    /// `+`
    Positive,
    /// `-`
    Negative,
    /// `!`
    Not,
}

/// Binary operations.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
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
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum Ref {
    /// A local variable.
    /// It is only accessible in current program.
    Var(String),
    /// A context variable.
    /// It is stored in the context.
    /// The variable name is prefixed with `$`.
    Ctx(String),
    /// A resource constant.
    /// The constant name is prefixed with `#`.
    Res(String),
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
        assert_eq!(RefParser::new().parse("#a").unwrap(), Ref::Res("a".into()));
    }
}
