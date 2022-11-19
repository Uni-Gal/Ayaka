#![feature(fn_traits)]
#![feature(tuple_trait)]
#![feature(unboxed_closures)]

use ayaka_bindings_types::*;
use ayaka_plugin::*;
use ayaka_script::log;
use scopeguard::defer;
use serde::{de::DeserializeOwned, Serialize};
use std::{marker::Tuple, path::Path};
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

pub struct WasmerModule {
    instance: Instance,
    memory: Memory,
    abi_free: NativeFunc<(i32, i32), ()>,
    abi_alloc: NativeFunc<i32, i32>,
}

impl WasmerModule {
    /// Loads the WASM [`Module`], with some imports.
    pub fn new(module: &Module, resolver: &(dyn Resolver + Send + Sync)) -> Result<Self> {
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

    fn call<T>(&self, name: &str, data: &[u8], f: impl FnOnce(&[u8]) -> Result<T>) -> Result<T> {
        let memory = &self.memory;
        let func = self
            .instance
            .exports
            .get_native_function::<(i32, i32), u64>(name)?;

        let ptr = self.abi_alloc.call(data.len() as i32)?;
        defer! { self.abi_free.call(ptr, data.len() as i32).unwrap(); }
        unsafe { mem_slice_mut(memory, ptr, data.len() as i32) }.copy_from_slice(&data);

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
    #[wasmer(export(name = "__abi_alloc"))]
    alloc: LazyInit<NativeFunc<i32, i32>>,
}

impl RuntimeInstanceData {
    pub unsafe fn import<Params: DeserializeOwned + Tuple, Res: Serialize>(
        &self,
        len: i32,
        data: i32,
        f: impl FnOnce<Params, Output = Res>,
    ) -> std::result::Result<u64, RuntimeError> {
        let memory = self.memory.get_unchecked();
        let data = mem_slice(memory, data, len);
        let data = rmp_serde::from_slice(data).map_err(|e| RuntimeError::new(e.to_string()))?;
        let res = f.call_once(data);
        let data = rmp_serde::to_vec(&res).map_err(|e| RuntimeError::new(e.to_string()))?;
        let alloc = self.alloc.get_unchecked();
        let ptr = alloc.call(data.len() as _)?;
        mem_slice_mut(memory, ptr, data.len() as _).copy_from_slice(&data);
        Ok(((data.len() as u64) << 32) | (ptr as u64))
    }
}

pub struct WasmerStoreLinker {
    store: Store,
    imports: Box<dyn NamedResolver + Send + Sync>,
}

impl WasmerStoreLinker {
    fn imports(
        store: &Store,
        root_path: impl AsRef<Path>,
    ) -> Result<Box<dyn NamedResolver + Send + Sync>> {
        let log_func = Function::new_native_with_env(
            store,
            RuntimeInstanceData::default(),
            |env_data: &RuntimeInstanceData, len: i32, data: i32| unsafe {
                env_data.import(len, data, |data: Record| {
                    log::logger().log(
                        &log::Record::builder()
                            .level(data.level)
                            .target(&data.target)
                            .args(format_args!("{}", data.msg))
                            .module_path(data.module_path.as_deref())
                            .file(data.file.as_deref())
                            .line(data.line)
                            .build(),
                    );
                })
            },
        );
        let log_flush_func = Function::new_native(store, || log::logger().flush());

        let import_object = imports! {
            "log" => {
                "__log" => log_func,
                "__log_flush" => log_flush_func,
            }
        };
        let wasi_env = WasiState::new("ayaka-runtime")
            .preopen_dir(root_path)?
            .finalize()?;
        let wasi_import = generate_import_object_from_env(store, wasi_env, WasiVersion::Latest);
        Ok(Box::new(import_object.chain_front(wasi_import)))
    }
}

impl StoreLinker<WasmerModule> for WasmerStoreLinker {
    fn new(root_path: impl AsRef<Path>) -> Result<Self> {
        let store = Store::default();
        let imports = Self::imports(&store, root_path)?;
        Ok(Self { store, imports })
    }

    fn from_binary(&self, binary: &[u8]) -> Result<WasmerModule> {
        let module = Module::from_binary(&self.store, binary)?;
        let host = WasmerModule::new(&module, &self.imports)?;
        Ok(host)
    }
}
