//! Wasmer-based plugin backend.

#![allow(clippy::mut_from_ref)]
#![warn(missing_docs)]

use ayaka_plugin::*;
use scopeguard::defer;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    path::Path,
};
use wasmer::*;
use wasmer_wasi::*;

unsafe fn mem_slice(memory: &Memory, start: i32, len: i32) -> &[u8] {
    memory
        .data_unchecked()
        .get_unchecked(start as usize..)
        .get_unchecked(..len as usize)
}

unsafe fn mem_slice_mut(memory: &Memory, start: i32, len: i32) -> &mut [u8] {
    memory
        .data_unchecked_mut()
        .get_unchecked_mut(start as usize..)
        .get_unchecked_mut(..len as usize)
}

/// A Wasmer [`Instance`].
pub struct WasmerModule {
    instance: Instance,
    memory: Memory,
    abi_free: NativeFunc<(i32, i32), ()>,
    abi_alloc: NativeFunc<i32, i32>,
}

impl WasmerModule {
    /// Loads the WASM [`Module`], with some imports.
    pub(crate) fn new(module: &Module, resolver: &(dyn Resolver + Send + Sync)) -> Result<Self> {
        let instance = Instance::new(module, resolver)?;
        let memory = instance.exports.get_memory("memory")?.clone();
        let abi_free = instance.exports.get_native_function("__abi_free")?;
        let abi_alloc = instance.exports.get_native_function("__abi_alloc")?;
        Ok(Self {
            instance,
            memory,
            abi_free,
            abi_alloc,
        })
    }
}

impl RawModule for WasmerModule {
    type Linker = WasmerStoreLinker;

    type Func = Function;

    fn call<T>(&self, name: &str, data: &[u8], f: impl FnOnce(&[u8]) -> Result<T>) -> Result<T> {
        let memory = &self.memory;
        let func = self
            .instance
            .exports
            .get_native_function::<(i32, i32), u64>(name)?;

        let ptr = self.abi_alloc.call(data.len() as i32)?;
        defer! { self.abi_free.call(ptr, data.len() as i32).unwrap(); }
        unsafe { mem_slice_mut(memory, ptr, data.len() as i32) }.copy_from_slice(data);

        let res = func.call(data.len() as i32, ptr)?;
        let (len, res) = ((res >> 32) as i32, (res & 0xFFFFFFFF) as i32);
        defer! { self.abi_free.call(res, len).unwrap(); }

        let res_data = unsafe { mem_slice(memory, res, len) };
        f(res_data)
    }
}

#[derive(Default, Clone, WasmerEnv)]
struct RuntimeInstanceData {
    #[wasmer(export)]
    memory: LazyInit<Memory>,
}

impl RuntimeInstanceData {
    pub unsafe fn import(
        &self,
        len: i32,
        data: i32,
        f: impl Fn(*const [u8]) -> Result<()>,
    ) -> std::result::Result<(), RuntimeError> {
        let memory = self.memory.get_unchecked();
        let data = mem_slice(memory, data, len);
        f(data).map_err(|e| RuntimeError::new(e.to_string()))?;
        Ok(())
    }
}

struct WasmerImportObjects(Vec<ImportObject>);

impl NamedResolver for WasmerImportObjects {
    fn resolve_by_name(&self, module: &str, field: &str) -> Option<Export> {
        let mut result = None;
        for imp in &self.0 {
            result = imp.resolve_by_name(module, field);
            if result.is_some() {
                return result;
            }
        }
        result
    }
}

impl Deref for WasmerImportObjects {
    type Target = Vec<ImportObject>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WasmerImportObjects {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A Wasmer [`Store`] with some [`ImportObject`]s.
pub struct WasmerStoreLinker {
    store: Store,
    imports: WasmerImportObjects,
}

impl StoreLinker<WasmerModule> for WasmerStoreLinker {
    fn new(root_path: impl AsRef<Path>) -> Result<Self> {
        let store = Store::default();
        let wasi_env = WasiState::new("ayaka-runtime")
            .preopen_dir(root_path)?
            .finalize()?;
        let wasi_import = generate_import_object_from_env(&store, wasi_env, WasiVersion::Latest);
        Ok(Self {
            store,
            imports: WasmerImportObjects(vec![wasi_import]),
        })
    }

    fn create(&self, binary: &[u8]) -> Result<WasmerModule> {
        let module = Module::from_binary(&self.store, binary)?;
        let host = WasmerModule::new(&module, &self.imports)?;
        Ok(host)
    }

    fn import(&mut self, ns: impl Into<String>, funcs: HashMap<String, Function>) -> Result<()> {
        let mut import_object = ImportObject::new();
        let mut namespace = Exports::new();
        for (name, func) in funcs {
            namespace.insert(name, func);
        }
        import_object.register(ns, namespace);
        self.imports.push(import_object);
        Ok(())
    }

    fn wrap(&self, f: impl Fn() + Send + Sync + 'static) -> Function {
        Function::new_native(&self.store, f)
    }

    fn wrap_with_args_raw(
        &self,
        f: impl (Fn(*const [u8]) -> Result<()>) + Send + Sync + 'static,
    ) -> Function {
        Function::new_native_with_env(
            &self.store,
            RuntimeInstanceData::default(),
            move |env_data: &RuntimeInstanceData, len: i32, data: i32| unsafe {
                env_data.import(len, data, &f)
            },
        )
    }
}
