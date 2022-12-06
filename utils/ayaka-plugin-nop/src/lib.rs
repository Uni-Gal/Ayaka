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
    type Linker = NopLinker;

    type LinkerHandle<'a> = NopLinker;

    type Func = ();

    fn call<T>(&self, name: &str, _args: &[u8], _f: impl FnOnce(&[u8]) -> Result<T>) -> Result<T> {
        bail!("Trying to call {name}.")
    }
}

/// A no-operation store & linker.
pub struct NopLinker;

impl Linker<NopModule> for NopLinker {
    fn new(_root_path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self)
    }

    fn create(&self, _binary: &[u8]) -> Result<NopModule> {
        Ok(NopModule)
    }

    fn import(&mut self, _ns: impl Into<String>, _funcs: HashMap<String, ()>) -> Result<()> {
        Ok(())
    }

    fn wrap_raw(&self, _f: impl (Fn(Self, i32, i32) -> Result<Vec<u8>>) + Send + Sync + 'static) {}
}

impl<'a> LinkerHandle<'a, NopModule> for NopLinker {
    fn call<T>(
        &mut self,
        _m: &NopModule,
        name: &str,
        _args: &[u8],
        _f: impl FnOnce(&[u8]) -> Result<T>,
    ) -> Result<T> {
        bail!("Trying to call {name}.")
    }

    fn slice<T>(&self, _start: i32, _len: i32, _f: impl FnOnce(&[u8]) -> T) -> T {
        unimplemented!("Trying to slice.")
    }
}
