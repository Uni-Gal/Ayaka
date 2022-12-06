#![feature(once_cell)]

mod plugin;
mod script;

use ayaka_bindings::*;
use ayaka_bindings_types::VarMap;
use ayaka_script::*;
use plugin::*;
use script::*;
use std::sync::LazyLock;
use trylog::macros::*;

#[export]
fn plugin_type() -> PluginType {
    PluginType::builder().line(["exec"]).build()
}

static RUNTIME: LazyLock<Runtime> = LazyLock::new(Runtime::new);

#[export]
fn exec(mut ctx: LineProcessContext) -> LineProcessResult {
    let program = ctx.props["exec"].get_str();
    unwrap_or_default_log!(
        program.parse::<Program>().map(|exec| {
            let mut table = VarTable::new(&RUNTIME, &mut ctx.ctx.locals);
            table.call(&exec);
            LineProcessResult {
                locals: ctx.ctx.locals,
                vars: VarMap::default(),
            }
        }),
        "Cannot parse program"
    )
}
