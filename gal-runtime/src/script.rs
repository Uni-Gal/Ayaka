use crate::*;
use gal_script::*;

pub trait Callable {
    fn call(&self, ctx: &mut VarTable) -> Value;
}

impl Callable for Program {
    fn call(&self, ctx: &mut VarTable) -> Value {
        ctx.locals.clear();
        let mut res = Value::Unit;
        for expr in &self.0 {
            res = expr.call(ctx);
        }
        res
    }
}

impl Callable for Expr {
    fn call(&self, ctx: &mut VarTable) -> Value {
        match self {
            Self::Ref(r) => r.call(ctx),
            Self::Const(c) => c.call(ctx),
            Self::Assign(r, e) => {
                let val = e.call(ctx);
                assign(ctx, r, val);
                Value::Unit
            }
            Self::Call(n, args) => call(ctx, n, args),
        }
    }
}

fn assign(ctx: &mut VarTable, r: &Ref, val: Value) {
    match r {
        Ref::Var(n) => ctx.vars.insert(n.into(), val),
        Ref::Ctx(n) => ctx.locals.insert(n.into(), val),
        Ref::Res(_) => unimplemented!("Resources"),
    };
}

fn call(ctx: &mut VarTable, n: &str, args: &[Expr]) -> Value {
    match n {
        "if" => if args
            .get(0)
            .map(|e| e.call(ctx))
            .unwrap_or_default()
            .eval_bool(ctx)
        {
            args.get(1)
        } else {
            args.get(2)
        }
        .map(|e| e.call(ctx))
        .unwrap_or_default(),
        _ => unimplemented!("Call functions"),
    }
}

impl Callable for Ref {
    fn call(&self, ctx: &mut VarTable) -> Value {
        match self {
            Self::Var(n) => ctx.vars.get(n).map(|v| v.clone()).unwrap_or_default(),
            Self::Ctx(n) => ctx.locals.get(n).map(|v| v.clone()).unwrap_or_default(),
            Self::Res(_) => unimplemented!("Resources"),
        }
    }
}

impl Callable for Const {
    fn call(&self, _ctx: &mut VarTable) -> Value {
        match self {
            Self::Bool(b) => Value::Bool(*b),
            Self::Num(n) => Value::Num(*n),
            Self::Str(s) => Value::Str(s.clone()),
        }
    }
}

pub trait Evaluable {
    fn eval(&self, ctx: &mut VarTable) -> Value;

    fn eval_bool(&self, ctx: &mut VarTable) -> bool {
        match self.eval(ctx) {
            Value::Unit => false,
            Value::Bool(b) => b,
            Value::Num(i) => i != 0,
            Value::Str(s) => !s.is_empty(),
            Value::Expr(_) => unreachable!(),
        }
    }

    fn eval_str(&self, ctx: &mut VarTable) -> String {
        match self.eval(ctx) {
            Value::Unit => String::default(),
            Value::Bool(b) => b.to_string(),
            Value::Num(i) => i.to_string(),
            Value::Str(s) => s.clone(),
            Value::Expr(_) => unreachable!(),
        }
    }
}

impl Evaluable for Value {
    fn eval(&self, ctx: &mut VarTable) -> Value {
        match self {
            Value::Expr(p) => p.call(ctx),
            _ => self.clone(),
        }
    }
}
