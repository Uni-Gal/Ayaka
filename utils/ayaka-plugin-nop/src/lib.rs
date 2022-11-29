//! A no-operation plugin backend.

#![warn(missing_docs)]

use anyhow::bail;
use ayaka_plugin::*;
use std::{collections::HashMap, path::Path};

/// A no-operation WASM module.
///
/// Any call will return an error.
pub struct NopModule;

impl RawModule for NopModule {
    type Linker = NopStoreLinker;

    type Func = ();

    fn call<T>(&self, name: &str, _args: &[u8], _f: impl FnOnce(&[u8]) -> Result<T>) -> Result<T> {
        bail!("Trying to call {name}.")
    }
}

/// A no-operation store & linker.
pub struct NopStoreLinker;

impl StoreLinker<NopModule> for NopStoreLinker {
    fn new(_root_path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self)
    }

    fn create(&self, _binary: &[u8]) -> Result<NopModule> {
        Ok(NopModule)
    }

    fn import(&mut self, _ns: impl Into<String>, _funcs: HashMap<String, ()>) -> Result<()> {
        Ok(())
    }

    fn wrap(&self, _f: impl Fn() + Send + Sync + 'static) {}

    fn wrap_with_args_raw(&self, _f: impl (Fn(&[u8]) -> Result<()>) + Send + Sync + 'static) {}
}
