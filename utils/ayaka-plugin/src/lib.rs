//! Base crate for plugin runtimes.
//!
//! This crate provides abstract types and traits
//! for different plugin backends.

#![feature(fn_traits)]
#![feature(tuple_trait)]
#![feature(unboxed_closures)]
#![warn(missing_docs)]
#![deny(unsafe_code)]

pub use anyhow::Result;

use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashMap, marker::Tuple, path::Path};

/// The exported method `__abi_alloc`.
pub const ABI_ALLOC_NAME: &str = "__abi_alloc";
/// The exported method `__abi_free`.
pub const ABI_FREE_NAME: &str = "__abi_free";
/// The default exported memory name.
pub const MEMORY_NAME: &str = "memory";

/// Represents a raw plugin module.
pub trait RawModule: Sized {
    /// The linker type that can create raw module.
    type Linker: StoreLinker<Self>;

    /// The import function type.
    type Func;

    /// Calls a method by name.
    ///
    /// The args and returns are bytes.
    fn call<T>(&self, name: &str, args: &[u8], f: impl FnOnce(&[u8]) -> Result<T>) -> Result<T>;
}

/// High-level wrapper for plugin module.
pub struct PluginModule<M: RawModule> {
    module: M,
}

impl<M: RawModule> PluginModule<M> {
    /// Creates a wrapper on raw module.
    pub fn new(module: M) -> Self {
        Self { module }
    }

    #[doc(hidden)]
    pub fn call_raw(&self, name: &str, args: &[u8]) -> Result<Vec<u8>> {
        self.module.call(name, args, |data| Ok(data.to_vec()))
    }

    /// Call a method by name.
    ///
    /// The args and returns are passed by MessagePack with [`rmp_serde`].
    pub fn call<P: Serialize, R: DeserializeOwned>(&self, name: &str, args: P) -> Result<R> {
        let data = rmp_serde::to_vec(&args)?;
        self.module.call(name, &data, |res| {
            let res = rmp_serde::from_slice(res)?;
            Ok(res)
        })
    }
}

/// Represents the store & linker of plugin modules.
pub trait StoreLinker<M: RawModule>: Sized {
    /// Creates a new instance of [`StoreLinker`].
    ///
    /// The `root_path` is used to preopen the root dir,
    /// and mapped to `/`.
    fn new(root_path: impl AsRef<Path>) -> Result<Self>;

    /// Create a raw module from binary.
    fn create(&self, binary: &[u8]) -> Result<M>;

    /// Import functions by namespace and names.
    fn import(&mut self, ns: impl Into<String>, funcs: HashMap<String, M::Func>) -> Result<()>;

    /// Wrap a function with args in bytes.
    fn wrap_raw(&self, f: impl (Fn(&[u8]) -> Result<Vec<u8>>) + Send + Sync + 'static) -> M::Func;

    /// Wrap a function with args.
    fn wrap<P: DeserializeOwned + Tuple, R: Serialize>(
        &self,
        f: impl Fn<P, Output = R> + Send + Sync + 'static,
    ) -> M::Func {
        self.wrap_raw(move |data| {
            let data = rmp_serde::from_slice(data)?;
            let data = f.call(data);
            let data = rmp_serde::to_vec(&data)?;
            Ok(data)
        })
    }
}
