use ayaka_plugin::*;
use ayaka_script::*;
use std::collections::HashMap;

pub struct HostModule {
    name: String,
}

impl RawModule for HostModule {
    type Linker = HostLinker;

    type Func = ();

    fn call<T>(&self, name: &str, args: &[u8], f: impl FnOnce(&[u8]) -> Result<T>) -> Result<T> {
        todo!()
    }
}

pub struct HostLinker;

impl StoreLinker<HostModule> for HostLinker {
    fn new(_root_path: impl AsRef<std::path::Path>) -> Result<Self> {
        unimplemented!()
    }

    fn create(&self, _binary: &[u8]) -> Result<HostModule> {
        unimplemented!()
    }

    fn import(&mut self, _ns: impl Into<String>, _funcs: HashMap<String, ()>) -> Result<()> {
        Ok(())
    }

    fn wrap(&self, _f: impl Fn() + Send + Sync + 'static) {}

    fn wrap_with_args_raw(&self, _f: impl (Fn(&[u8]) -> Result<()>) + Send + Sync + 'static) {}
}

pub struct Module {
    module: PluginModule<HostModule>,
}

impl Module {
    fn new(module: HostModule) -> Self {
        Self {
            module: PluginModule::new(module),
        }
    }

    /// Calls a script plugin method by name.
    pub fn dispatch_method(&self, name: &str, args: &[RawValue]) -> Result<RawValue> {
        self.module.call(name, (args,))
    }
}

pub struct Runtime {
    modules: HashMap<String, Module>,
}

impl Runtime {
    /// Gets module from name.
    pub fn module(&self, key: &str) -> Option<&Module> {
        self.modules.get(key)
    }
}
