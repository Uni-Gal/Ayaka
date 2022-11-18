use anyhow::bail;
use ayaka_plugin::*;

pub struct NopModule;

impl RawModule for NopModule {
    type Linker = NopStoreLinker;

    type Func = ();

    fn call<T>(&self, name: &str, _args: &[u8], _f: impl FnOnce(&[u8]) -> Result<T>) -> Result<T> {
        bail!("Trying to call {name}.")
    }
}

pub struct NopStoreLinker;

impl StoreLinker<NopModule> for NopStoreLinker {
    fn new(_root_path: impl AsRef<std::path::Path>) -> Result<Self> {
        Ok(Self)
    }

    fn create(&self, _binary: &[u8]) -> Result<NopModule> {
        Ok(NopModule)
    }

    fn import(
        &mut self,
        _ns: impl Into<String>,
        _funcs: std::collections::HashMap<String, ()>,
    ) -> Result<()> {
        Ok(())
    }

    fn wrap(&self, _f: impl Fn() + Send + Sync + 'static) {}

    fn wrap_with_args_raw(&self, _f: impl (Fn(*const [u8]) -> Result<()>) + Send + Sync + 'static) {
    }
}