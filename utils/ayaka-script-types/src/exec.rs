use crate::*;
use serde::{Deserialize, Serialize};

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
