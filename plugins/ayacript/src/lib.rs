#![deny(unsafe_code)]
#![feature(once_cell)]

mod plugin;
mod script;

use ayaka_bindings::*;
use ayaka_bindings_types::VarMap;
use ayaka_script::*;
use plugin::*;
use script::*;
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};
use trylog::macros::*;

#[export]
fn plugin_type() -> PluginType {
    PluginType::builder().line(["exec"]).build()
}

static RUNTIME: LazyLock<Runtime> = LazyLock::new(Runtime::new);
static PROGRAM_CACHE: LazyLock<Mutex<HashMap<LineKey, Program>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
struct LineKey {
    pub cur_base_para: String,
    pub cur_para: String,
    pub cur_act: usize,
}

#[export]
fn exec(mut ctx: LineProcessContext) -> LineProcessResult {
    let key = LineKey {
        cur_base_para: ctx.ctx.cur_base_para,
        cur_para: ctx.ctx.cur_para,
        cur_act: ctx.ctx.cur_act,
    };
    let mut cache = PROGRAM_CACHE.lock().unwrap();
    let exec = cache.entry(key).or_insert_with(|| {
        let program = ctx.props["exec"].get_str();
        unwrap_or_default_log!(program.parse::<Program>(), "Cannot parse program")
    });
    let mut table = VarTable::new(&RUNTIME, &mut ctx.ctx.locals);
    table.call(exec);
    let vars = table.vars;
    LineProcessResult {
        locals: ctx.ctx.locals,
        vars,
    }
}
