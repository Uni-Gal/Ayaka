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
    fn new(name: String) -> Self {
        Self { name }
    }
}

impl PluginModule for Proxy {
    fn call<P: Serialize, R: DeserializeOwned>(&self, name: &str, args: P) -> Result<R> {
        let args = rmp_serde::to_vec(&args)?;
        let res = __call(&self.name, name, &args);
        Ok(rmp_serde::from_slice(&res)?)
    }
}

pub struct Runtime(PluginRuntime<Proxy>);

impl Runtime {
    pub fn load() -> Result<Self> {
        let mut inner = PluginRuntime::<Proxy>::new();
        for name in __modules() {
            inner.insert_module(name.clone(), Proxy::new(name))?;
        }
        Ok(Self(inner))
    }
}
