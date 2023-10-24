use crate::*;
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(
    #[allow(missing_docs)]
    #[allow(dead_code)]
    #[allow(clippy::all)]
    #[allow(unused_imports)]
    grammer
);

pub use grammer::{ConstParser, ExprParser, ProgramParser, RefParser};

impl std::str::FromStr for Program {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ProgramParser::new()
            .parse(s)
            .map_err(|e| anyhow::anyhow!("{}", e))
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
