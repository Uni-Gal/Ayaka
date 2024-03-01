use crate::*;
use ayaka_plugin::*;
use ayaka_script::*;

#[import("plugin")]
extern "C" {
    fn __modules() -> Vec<String>;
    fn __call(module: &str, name: &str, args: &[u8]) -> Vec<u8>;
}

pub struct HostModule {
    name: String,
}

impl RawModule for HostModule {
    type Linker = HostLinker;

    type LinkerHandle<'a> = HostLinker;

    type Func = ();

    fn call<T>(&self, name: &str, args: &[u8], f: impl FnOnce(&[u8]) -> Result<T>) -> Result<T> {
        let data = __call(&self.name, name, args);
        f(&data)
    }
}

pub struct HostLinker;

impl Linker<HostModule> for HostLinker {
    type Config = ();

    fn new(_: ()) -> Result<Self> {
        unimplemented!()
    }

    fn create(&self, _binary: &[u8]) -> Result<HostModule> {
        unimplemented!()
    }

    fn import(&mut self, _ns: impl Into<String>, _funcs: HashMap<String, ()>) -> Result<()> {
        Ok(())
    }

    fn wrap_raw(&self, _f: impl (Fn(Self, i32, i32) -> Result<Vec<u8>>) + Send + Sync + 'static) {}
}

impl<'a> LinkerHandle<'a, HostModule> for HostLinker {
    fn call<T>(
        &mut self,
        _m: &HostModule,
        _name: &str,
        _args: &[u8],
        _f: impl FnOnce(&[u8]) -> Result<T>,
    ) -> Result<T> {
        unimplemented!()
    }

    fn slice<T>(&self, _start: i32, _len: i32, _f: impl FnOnce(&[u8]) -> T) -> T {
        unimplemented!()
    }

    fn slice_mut<T>(&mut self, _start: i32, _len: i32, _f: impl FnOnce(&mut [u8]) -> T) -> T {
        unimplemented!()
    }
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
    pub fn new() -> Self {
        let modules = __modules();
        let modules = modules
            .into_iter()
            .map(|name| {
                let m = HostModule { name: name.clone() };
                (name, Module::new(m))
            })
            .collect();
        Self { modules }
    }

    /// Gets module from name.
    pub fn module(&self, key: &str) -> Option<&Module> {
        self.modules.get(key)
    }
}
