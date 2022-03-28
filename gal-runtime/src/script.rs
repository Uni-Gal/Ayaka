use crate::*;
use gal_script::*;

pub trait Callable {
    fn call(&self, ctx: &mut VarTable) -> Value;
}

impl Callable for Program {
    fn call(&self, ctx: &mut VarTable) -> Value {
        ctx.vars.clear();
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
            Self::Unary(op, e) => match op {
                UnaryOp::Positive => Value::Num(e.call(ctx).eval_num(ctx)),
                UnaryOp::Negative => Value::Num(-e.call(ctx).eval_num(ctx)),
                UnaryOp::Not => match e.call(ctx) {
                    Value::Unit => Value::Unit,
                    Value::Bool(b) => Value::Bool(!b),
                    Value::Num(i) => Value::Num(!i),
                    Value::Str(_) => Value::Str(String::new()),
                    Value::Expr(_) => unimplemented!(),
                },
            },
            Self::Binary(lhs, op, rhs) => match op {
                BinaryOp::Val(op) => bin_val(ctx, lhs, op, rhs),
                BinaryOp::Logic(op) => bin_logic(ctx, lhs, op, rhs),
                BinaryOp::Assign => {
                    let val = rhs.call(ctx);
                    assign(ctx, lhs, val)
                }
                BinaryOp::Inplace(op) => {
                    let val = bin_val(ctx, lhs, op, rhs);
                    assign(ctx, lhs, val)
                }
            },
            Self::Call(n, args) => call(ctx, n, args),
        }
    }
}

fn bin_val(ctx: &mut VarTable, lhs: &Expr, op: &ValBinaryOp, rhs: &Expr) -> Value {
    unimplemented!()
}

fn bin_logic(ctx: &mut VarTable, lhs: &Expr, op: &LogicBinaryOp, rhs: &Expr) -> Value {
    let res = match op {
        LogicBinaryOp::Eq => lhs.call(ctx).eval_num(ctx) == rhs.call(ctx).eval_num(ctx),
        _ => unimplemented!(),
    };
    Value::Bool(res)
}

fn assign(ctx: &mut VarTable, e: &Expr, val: Value) -> Value {
    match e {
        Expr::Ref(r) => match r {
            Ref::Var(n) => ctx.vars.insert(n.into(), val),
            Ref::Ctx(n) => ctx.locals.insert(n.into(), val),
            Ref::Res(_) => unimplemented!("Resources"),
        },
        _ => unreachable!(),
    };
    Value::Unit
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

    fn eval_num(&self, ctx: &mut VarTable) -> i64 {
        match self.eval(ctx) {
            Value::Unit => 0,
            Value::Bool(b) => {
                if b {
                    1
                } else {
                    0
                }
            }
            Value::Num(i) => i,
            Value::Str(s) => s.len() as i64,
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
