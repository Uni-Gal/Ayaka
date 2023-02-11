//! Base crate for plugin runtimes.
//!
//! This crate provides abstract types and traits
//! for different plugin backends.

#![feature(tuple_trait)]
#![warn(missing_docs)]
#![deny(unsafe_code)]

#[doc(no_inline)]
pub use anyhow::Result;

use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashMap, marker::Tuple};

/// The exported method `__abi_alloc`.
pub const ABI_ALLOC_NAME: &str = "__abi_alloc";
/// The exported method `__abi_free`.
pub const ABI_FREE_NAME: &str = "__abi_free";
/// The default exported memory name.
pub const MEMORY_NAME: &str = "memory";

/// Represents a raw plugin module.
pub trait RawModule: Sized {
    /// The linker type that can create raw module.
    type Linker: Linker<Self>;

    /// The linker handle type.
    type LinkerHandle<'a>: LinkerHandle<'a, Self>;

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

    /// Get inner raw module.
    pub fn inner(&self) -> &M {
        &self.module
    }
}

/// Represents the linker of plugin modules.
pub trait Linker<M: RawModule>: Sized {
    /// Create the linker.
    fn new() -> Result<Self>;

    /// Create a raw module from binary.
    fn create(&self, binary: &[u8]) -> Result<M>;

    /// Import functions by namespace and names.
    fn import(&mut self, ns: impl Into<String>, funcs: HashMap<String, M::Func>) -> Result<()>;

    /// Wrap a function with args in bytes.
    fn wrap_raw(
        &self,
        f: impl (Fn(M::LinkerHandle<'_>, i32, i32) -> Result<Vec<u8>>) + Send + Sync + 'static,
    ) -> M::Func;

    /// Wrap a function with args.
    fn wrap<P: DeserializeOwned + Tuple, R: Serialize>(
        &self,
        f: impl (Fn(P) -> Result<R>) + Send + Sync + 'static,
    ) -> M::Func {
        self.wrap_raw(move |handle, start, len| {
            let data = handle.slice(start, len, |data| rmp_serde::from_slice(data))?;
            let data = f(data)?;
            let data = rmp_serde::to_vec(&data)?;
            Ok(data)
        })
    }

    /// Wrap a function with args and linker handle.
    fn wrap_with<P: DeserializeOwned + Tuple, R: Serialize>(
        &self,
        f: impl (Fn(M::LinkerHandle<'_>, P) -> Result<R>) + Send + Sync + 'static,
    ) -> M::Func {
        self.wrap_raw(move |handle, start, len| {
            let data = handle.slice(start, len, |data| rmp_serde::from_slice(data))?;
            let data = f(handle, data)?;
            let data = rmp_serde::to_vec(&data)?;
            Ok(data)
        })
    }
}

/// Represents a handle of linker.
///
/// Usually it is not easy to call function inside host functions.
/// A handle provides methods to do so.
pub trait LinkerHandle<'a, M: RawModule> {
    /// Call methods of a module.
    fn call<T>(
        &mut self,
        m: &M,
        name: &str,
        args: &[u8],
        f: impl FnOnce(&[u8]) -> Result<T>,
    ) -> Result<T>;

    /// Get memory slice.
    fn slice<T>(&self, start: i32, len: i32, f: impl FnOnce(&[u8]) -> T) -> T;

    /// Get memory mutable slice.
    fn slice_mut<T>(&mut self, start: i32, len: i32, f: impl FnOnce(&mut [u8]) -> T) -> T;
}
