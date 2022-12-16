use super::Runtime;
use anyhow::{bail, Result};
use ayaka_plugin::{Linker, LinkerHandle, RawModule};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, Weak},
};

pub fn register<M: RawModule + Send + Sync + 'static>(
    store: &mut impl Linker<M>,
    handle: Arc<RwLock<Weak<Runtime<M>>>>,
) -> Result<()> {
    let h = handle.clone();
    let modules_func = store.wrap(move |_: ()| {
        if let Some(this) = h.read().unwrap().upgrade() {
            Ok(this.modules.keys().cloned().collect::<Vec<_>>())
        } else {
            bail!("Runtime hasn't been initialized.")
        }
    });
    let h = handle;
    let call_func = store.wrap_with(
        move |mut handle, (module, name, args): (String, String, Vec<u8>)| {
            if let Some(this) = h.read().unwrap().upgrade() {
                Ok(handle.call(
                    this.modules[&module].module.inner(),
                    &name,
                    &args,
                    |slice| Ok(slice.to_vec()),
                )?)
            } else {
                bail!("Runtime hasn't been initialized.")
            }
        },
    );
    store.import(
        "plugin",
        HashMap::from([
            ("__modules".to_string(), modules_func),
            ("__call".to_string(), call_func),
        ]),
    )?;
    Ok(())
}
