//! The script parsers.

#![warn(missing_docs)]
#![deny(unsafe_code)]

#[cfg(feature = "parser")]
mod parser;
#[cfg(feature = "parser")]
pub use parser::*;

#[doc(no_inline)]
pub use ayaka_primitive::{RawValue, ValueType};

use serde::{Deserialize, Serialize};

/// A full script, a collection of expressions.
///
/// The last expression is the final value of the script.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Program(pub Vec<Expr>);

/// An expression.
///
/// Two expressions should be splited with `;`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    /// `+`
    Positive,
    /// `-`
    Negative,
    /// `!`
    Not,
}

/// Binary operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ref {
    /// A local variable.
    /// It is only accessible in current program.
    Var(String),
    /// A context variable.
    /// It is stored in the context.
    /// The variable name is prefixed with `$`.
    Ctx(String),
}
