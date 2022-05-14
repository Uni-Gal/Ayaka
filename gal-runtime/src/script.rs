use crate::*;
use gal_script::*;

pub struct VarTable<'a> {
    pub locals: &'a mut VarMap,
    pub res: &'a VarMap,
    pub runtime: &'a mut Runtime,
    pub vars: VarMap,
}

impl<'a> VarTable<'a> {
    pub fn new(locals: &'a mut VarMap, res: &'a VarMap, runtime: &'a mut Runtime) -> Self {
        Self {
            locals,
            res,
            runtime,
            vars: VarMap::default(),
        }
    }

    pub fn call(&mut self, c: &impl Callable) -> RawValue {
        c.call(self)
    }
}

pub trait Callable {
    fn call(&self, ctx: &mut VarTable) -> RawValue;
}

impl<T: Callable> Callable for Option<&T> {
    fn call(&self, ctx: &mut VarTable) -> RawValue {
        match self {
            Some(c) => c.call(ctx),
            None => RawValue::Unit,
        }
    }
}

impl Callable for Program {
    fn call(&self, ctx: &mut VarTable) -> RawValue {
        ctx.vars.clear();
        let mut res = RawValue::Unit;
        for expr in &self.0 {
            res = expr.call(ctx);
        }
        res
    }
}

impl Callable for Expr {
    fn call(&self, ctx: &mut VarTable) -> RawValue {
        match self {
            Self::Ref(r) => r.call(ctx),
            Self::Const(c) => c.clone(),
            Self::Unary(op, e) => match op {
                UnaryOp::Positive => RawValue::Num(e.call(ctx).get_num()),
                UnaryOp::Negative => RawValue::Num(-e.call(ctx).get_num()),
                UnaryOp::Not => match e.call(ctx) {
                    RawValue::Unit => RawValue::Unit,
                    RawValue::Bool(b) => RawValue::Bool(!b),
                    RawValue::Num(i) => RawValue::Num(!i),
                    RawValue::Str(_) => RawValue::Str(String::new()),
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
            Self::Call(ns, name, args) => call(ctx, ns, name, args),
        }
    }
}

fn bin_val(ctx: &mut VarTable, lhs: &Expr, op: &ValBinaryOp, rhs: &Expr) -> RawValue {
    let lhs = lhs.call(ctx);
    let rhs = rhs.call(ctx);
    let t = lhs.get_type().max(rhs.get_type());
    match t {
        ValueType::Unit => RawValue::Unit,
        ValueType::Bool => bin_bool_val(lhs.get_bool(), op, rhs.get_bool()),
        ValueType::Num => RawValue::Num(bin_num_val(lhs.get_num(), op, rhs.get_num())),
        ValueType::Str => bin_str_val(lhs, op, rhs),
    }
}

fn bin_bool_val(lhs: bool, op: &ValBinaryOp, rhs: bool) -> RawValue {
    match op {
        ValBinaryOp::Add
        | ValBinaryOp::Minus
        | ValBinaryOp::Mul
        | ValBinaryOp::Div
        | ValBinaryOp::Mod => RawValue::Num(bin_num_val(lhs as i64, op, rhs as i64)),
        ValBinaryOp::And => RawValue::Bool(lhs && rhs),
        ValBinaryOp::Or => RawValue::Bool(lhs || rhs),
        ValBinaryOp::Xor => RawValue::Bool(lhs ^ rhs),
    }
}

fn bin_num_val(lhs: i64, op: &ValBinaryOp, rhs: i64) -> i64 {
    match op {
        ValBinaryOp::Add => lhs + rhs,
        ValBinaryOp::Minus => lhs - rhs,
        ValBinaryOp::Mul => lhs * rhs,
        ValBinaryOp::Div => lhs / rhs,
        ValBinaryOp::Mod => lhs % rhs,
        ValBinaryOp::And => lhs & rhs,
        ValBinaryOp::Or => lhs | rhs,
        ValBinaryOp::Xor => lhs ^ rhs,
    }
}

fn bin_str_val(lhs: RawValue, op: &ValBinaryOp, rhs: RawValue) -> RawValue {
    match op {
        ValBinaryOp::Add => RawValue::Str((lhs.get_str() + rhs.get_str()).into()),
        ValBinaryOp::Mul => match (
            lhs.get_type().max(ValueType::Num),
            rhs.get_type().max(ValueType::Num),
        ) {
            (ValueType::Str, ValueType::Str) => unimplemented!(),
            (ValueType::Num, ValueType::Str) => {
                RawValue::Str(rhs.get_str().repeat(lhs.get_num() as usize))
            }
            (ValueType::Str, ValueType::Num) => {
                RawValue::Str(lhs.get_str().repeat(rhs.get_num() as usize))
            }
            _ => unreachable!(),
        },
        _ => unimplemented!(),
    }
}

fn bin_logic(ctx: &mut VarTable, lhs: &Expr, op: &LogicBinaryOp, rhs: &Expr) -> RawValue {
    let res = match op {
        LogicBinaryOp::And => lhs.call(ctx).get_bool() && rhs.call(ctx).get_bool(),
        LogicBinaryOp::Or => lhs.call(ctx).get_bool() || rhs.call(ctx).get_bool(),
        op => {
            let lhs = lhs.call(ctx);
            let rhs = rhs.call(ctx);
            let t = lhs.get_type().max(rhs.get_type());
            match t {
                ValueType::Unit => false,
                ValueType::Bool => bin_ord_logic(&lhs.get_bool(), op, &rhs.get_bool()),
                ValueType::Num => bin_ord_logic(&lhs.get_num(), op, &rhs.get_num()),
                ValueType::Str => bin_ord_logic(&lhs.get_str(), op, &rhs.get_str()),
            }
        }
    };
    RawValue::Bool(res)
}

fn bin_ord_logic<T: Ord>(lhs: &T, op: &LogicBinaryOp, rhs: &T) -> bool {
    match op {
        LogicBinaryOp::Eq => lhs == rhs,
        LogicBinaryOp::Neq => lhs != rhs,
        LogicBinaryOp::Lt => lhs < rhs,
        LogicBinaryOp::Le => lhs <= rhs,
        LogicBinaryOp::Gt => lhs > rhs,
        LogicBinaryOp::Ge => lhs >= rhs,
        _ => unreachable!(),
    }
}

fn assign(ctx: &mut VarTable, e: &Expr, val: RawValue) -> RawValue {
    match e {
        Expr::Ref(r) => match r {
            Ref::Var(n) => ctx.vars.insert(n.into(), val),
            Ref::Ctx(n) => ctx.locals.insert(n.into(), val),
            Ref::Res(_) => unimplemented!("Resources"),
        },
        _ => unreachable!(),
    };
    RawValue::Unit
}

fn call(ctx: &mut VarTable, ns: &str, name: &str, args: &[Expr]) -> RawValue {
    if ns.is_empty() {
        match name {
            "if" => if args.get(0).call(ctx).get_bool() {
                args.get(1)
            } else {
                args.get(2)
            }
            .call(ctx),
            _ => unimplemented!("intrinstics"),
        }
    } else {
        let args = args.iter().map(|e| e.call(ctx)).collect::<Vec<_>>();
        let runtime = ctx.runtime.modules.get(ns).unwrap();
        runtime
            .dispatch(&mut ctx.runtime.store, name, &args)
            .unwrap()
    }
}

impl Callable for Ref {
    fn call(&self, ctx: &mut VarTable) -> RawValue {
        match self {
            Self::Var(n) => ctx.vars.get(n).cloned().unwrap_or_default(),
            Self::Ctx(n) => ctx.locals.get(n).cloned().unwrap_or_default(),
            Self::Res(_) => unimplemented!("Resources"),
        }
    }
}

impl Callable for Text {
    fn call(&self, ctx: &mut VarTable) -> RawValue {
        let mut str = String::new();
        for line in &self.0 {
            match line {
                Line::Str(s) => str.push_str(s),
                Line::Cmd(c) => match c {
                    Command::Exec(p) => str.push_str(&p.call(ctx).get_str()),
                    _ => {}
                },
            }
        }
        RawValue::Str(str)
    }
}

#[cfg(test)]
mod test {
    use crate::*;
    use gal_script::*;
    use std::sync::Mutex;

    lazy_static::lazy_static! {
        static ref RUNTIME: Mutex<Runtime> = Mutex::new(load_plugins(
            "../target/wasm32-unknown-unknown/release/",
            env!("CARGO_MANIFEST_DIR"),
        ));
    }

    fn with_ctx(f: impl FnOnce(&mut VarTable)) {
        let mut locals = VarMap::default();
        let res = VarMap::default();
        let mut runtime = RUNTIME.lock().unwrap();
        let mut ctx = VarTable::new(&mut locals, &res, &mut runtime);
        f(&mut ctx);
    }

    #[test]
    fn vars() {
        with_ctx(|ctx| {
            assert_eq!(
                ProgramParser::new()
                    .parse(
                        "
                            a = 0;
                            a += 1;
                            a += a;
                            a
                        "
                    )
                    .unwrap()
                    .call(ctx),
                RawValue::Num(2)
            );

            assert_eq!(
                ProgramParser::new().parse("a").unwrap().call(ctx),
                RawValue::Unit
            );

            assert_eq!(
                ProgramParser::new()
                    .parse(
                        "
                            $a = 0;
                            $a += 1;
                            $a += a;
                            $a
                        "
                    )
                    .unwrap()
                    .call(ctx),
                RawValue::Num(1)
            );

            assert_eq!(
                ProgramParser::new().parse("$a").unwrap().call(ctx),
                RawValue::Num(1)
            );
        });
    }

    #[test]
    fn if_test() {
        with_ctx(|ctx| {
            assert_eq!(
                ProgramParser::new()
                    .parse(
                        r##"
                            if(1 + 1 + 4 + 5 + 1 + 4 == 16, "sodayo", ~)
                        "##
                    )
                    .unwrap()
                    .call(ctx)
                    .get_num(),
                6
            );
        });
    }

    #[test]
    fn format() {
        with_ctx(|ctx| {
            assert_eq!(
                ProgramParser::new()
                    .parse(
                        r##"
                            format.fmt("Hello {}!", 114514)
                        "##
                    )
                    .unwrap()
                    .call(ctx)
                    .get_str(),
                "Hello 114514!"
            )
        })
    }
}
