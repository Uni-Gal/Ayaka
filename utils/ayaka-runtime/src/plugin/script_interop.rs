use anyhow::Result;
use ayaka_plugin::{Linker, RawModule};
use ayaka_script::Program;
use std::collections::HashMap;

pub fn register<M: RawModule>(store: &mut impl Linker<M>) -> Result<()> {
    let parse_func = store.wrap(|(program,): (String,)| program.parse::<Program>());
    store.import(
        "script",
        HashMap::from([("__parse".to_string(), parse_func)]),
    )?;
    Ok(())
}
