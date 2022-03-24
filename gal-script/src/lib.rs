use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub gal);

#[derive(Debug, PartialEq, Eq)]
pub struct Program(pub Vec<Expr>);

#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    Ref(Ref),
    Const(Const),
    Call(String, Vec<Expr>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Ref {
    Var(String),
    Ctx(String),
    Res(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Const {
    Bool(bool),
    Num(i64),
    Str(String),
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
            gal::ProgramParser::new()
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
        assert_eq!(gal::ExprParser::new().parse("a").unwrap(), var("a"));
        assert_eq!(
            gal::ExprParser::new().parse("foo(a)").unwrap(),
            Expr::Call("foo".into(), vec![var("a")])
        );
        assert_eq!(
            gal::ExprParser::new().parse("foo(a, b)").unwrap(),
            Expr::Call("foo".into(), vec![var("a"), var("b")])
        );
    }

    #[test]
    fn r#const() {
        assert_eq!(
            gal::ConstParser::new().parse("true").unwrap(),
            Const::Bool(true)
        );
        assert_eq!(
            gal::ConstParser::new().parse("false").unwrap(),
            Const::Bool(false)
        );

        assert_eq!(
            gal::ConstParser::new().parse("114514").unwrap(),
            Const::Num(114514)
        );
        assert_eq!(
            gal::ConstParser::new().parse("-1919810").unwrap(),
            Const::Num(-1919810)
        );

        assert_eq!(
            gal::ConstParser::new().parse("\"Hello world!\"").unwrap(),
            Const::Str("Hello world!".into())
        );
    }

    #[test]
    fn r#ref() {
        assert_eq!(
            gal::RefParser::new().parse("a").unwrap(),
            Ref::Var("a".into())
        );
        assert_eq!(
            gal::RefParser::new().parse("$a").unwrap(),
            Ref::Ctx("a".into())
        );
        assert_eq!(
            gal::RefParser::new().parse("#a").unwrap(),
            Ref::Res("a".into())
        );
    }
}
