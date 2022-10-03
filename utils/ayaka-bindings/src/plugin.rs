use anyhow::Result;
use ayaka_bindings_impl::import;
use ayaka_bindings_types::*;
use serde::{de::DeserializeOwned, Serialize};

#[import("plugin")]
extern "C" {
    fn __modules() -> Vec<String>;
    fn __call(module: &str, name: &str, args: &[u8]) -> Vec<u8>;
}

pub struct Proxy {
    name: String,
}

impl Proxy {
    fn new(name: &[u8]) -> Self {
        Self {
            name: String::from_utf8_lossy(name).into_owned(),
        }
    }
}

impl PluginModule for Proxy {
    fn call<P: Serialize, R: DeserializeOwned>(&self, name: &str, args: P) -> Result<R> {
        let args = rmp_serde::to_vec(&args)?;
        let res = __call(&self.name, name, &args);
        Ok(rmp_serde::from_slice(&res)?)
    }
}

pub struct ProxyStore;

impl PluginModuleStore for ProxyStore {
    type Module = Proxy;

    fn from_binary(&self, binary: &[u8]) -> Result<Self::Module> {
        Ok(Proxy::new(binary))
    }
}

pub struct Runtime(PluginRuntime<Proxy, ProxyStore>);

impl Runtime {
    pub fn load() -> Result<Self> {
        let mut inner = PluginRuntime::new(ProxyStore);
        for name in __modules() {
            inner.insert_binary(name.clone(), name.as_bytes())?;
        }
        Ok(Self(inner))
    }
}
