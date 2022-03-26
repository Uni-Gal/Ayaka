use crate::*;
use gal_script::*;

pub trait Callable {
    fn call(&self, ctx: &mut VarTable) -> Option<Value>;
}

impl Callable for Program {
    fn call(&self, ctx: &mut VarTable) -> Option<Value> {
        ctx.locals.clear();
        let mut res = None;
        for expr in &self.0 {
            res = expr.call(ctx);
        }
        res
    }
}

impl Callable for Expr {
    fn call(&self, ctx: &mut VarTable) -> Option<Value> {
        match self {
            Self::Ref(r) => r.call(ctx),
            Self::Const(c) => c.call(ctx),
            Self::Assign(r, e) => {
                let val = e.call(ctx);
                assign(ctx, r, val);
                None
            }
            Self::Call(n, args) => call(ctx, n, args),
        }
    }
}

fn assign(ctx: &mut VarTable, r: &Ref, val: Option<Value>) {
    if let Some(val) = val {
        match r {
            Ref::Var(n) => ctx.vars.insert(n.into(), val),
            Ref::Ctx(n) => ctx.locals.insert(n.into(), val),
            Ref::Res(_) => unimplemented!("Resources"),
        }
    } else {
        match r {
            Ref::Var(n) => ctx.vars.remove(n),
            Ref::Ctx(n) => ctx.vars.remove(n),
            Ref::Res(_) => unreachable!(),
        }
    };
}

fn call(ctx: &mut VarTable, n: &str, args: &[Expr]) -> Option<Value> {
    match n {
        "if" => {
            if args
                .get(0)
                .and_then(|e| e.call(ctx))
                .map(|v| v.eval_bool(ctx))
                .unwrap_or_default()
            {
                args.get(1).and_then(|e| e.call(ctx))
            } else {
                args.get(2).and_then(|e| e.call(ctx))
            }
        }
        _ => unimplemented!("Call functions"),
    }
}

impl Callable for Ref {
    fn call(&self, ctx: &mut VarTable) -> Option<Value> {
        match self {
            Self::Var(n) => ctx.vars.get(n).map(|v| v.clone()),
            Self::Ctx(n) => ctx.locals.get(n).map(|v| v.clone()),
            Self::Res(_) => unimplemented!("Resources"),
        }
    }
}

impl Callable for Const {
    fn call(&self, _ctx: &mut VarTable) -> Option<Value> {
        match self {
            Self::Bool(b) => Some(Value::Bool(*b)),
            Self::Num(n) => Some(Value::Num(*n)),
            Self::Str(s) => Some(Value::Str(s.clone())),
        }
    }
}

pub trait Evaluable {
    fn eval(&self, ctx: &mut VarTable) -> Value;

    fn eval_bool(&self, ctx: &mut VarTable) -> bool {
        match self.eval(ctx) {
            Value::Bool(b) => b,
            Value::Num(i) => i != 0,
            Value::Str(s) => !s.is_empty(),
            Value::Expr(_) => unreachable!(),
        }
    }

    fn eval_str(&self, ctx: &mut VarTable) -> String {
        match self.eval(ctx) {
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
            Value::Expr(p) => p.call(ctx).unwrap_or(Value::Str(String::new())),
            _ => self.clone(),
        }
    }
}
