use crate::*;
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(grammer, "/exec/grammer.rs");

pub use grammer::*;

#[derive(Debug, Default, PartialEq, Eq)]
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
