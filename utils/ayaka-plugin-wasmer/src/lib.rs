//! Wasmer-based plugin backend.

#![allow(clippy::mut_from_ref)]
#![warn(missing_docs)]

use ayaka_plugin::*;
use scopeguard::defer;
use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};
use wasmer::*;
use wasmer_wasi::*;

unsafe fn mem_slice<R>(
    store: &impl AsStoreRef,
    memory: &Memory,
    start: i32,
    len: i32,
    f: impl FnOnce(&[u8]) -> R,
) -> R {
    f(memory
        .view(store)
        .data_unchecked()
        .get_unchecked(start as usize..)
        .get_unchecked(..len as usize))
}

unsafe fn mem_slice_mut<R>(
    store: &impl AsStoreRef,
    memory: &Memory,
    start: i32,
    len: i32,
    f: impl FnOnce(&mut [u8]) -> R,
) -> R {
    f(memory
        .view(store)
        .data_unchecked_mut()
        .get_unchecked_mut(start as usize..)
        .get_unchecked_mut(..len as usize))
}

type HostStore = Arc<Mutex<Store>>;

struct HostInstance {
    store: HostStore,
    instance: Instance,
}

impl HostInstance {
    pub fn new(store: HostStore, instance: Instance) -> Self {
        Self { store, instance }
    }

    pub fn get_memory(&self) -> Result<HostMemory, ExportError> {
        self.instance
            .exports
            .get_memory(MEMORY_NAME)
            .map(|mem| HostMemory::new(self.store.clone(), mem.clone()))
    }

    pub fn get_func(&self, name: &str) -> Result<HostFunction, ExportError> {
        self.instance
            .exports
            .get_function(name)
            .map(|func| HostFunction::new(self.store.clone(), func.clone()))
    }
}

struct HostMemory {
    store: HostStore,
    memory: Memory,
}

impl HostMemory {
    pub fn new(store: HostStore, memory: Memory) -> Self {
        Self { store, memory }
    }

    pub unsafe fn slice<R>(&self, start: i32, len: i32, f: impl FnOnce(&[u8]) -> R) -> R {
        let store = self.store.lock().unwrap();
        mem_slice(&store.as_store_ref(), &self.memory, start, len, f)
    }

    pub unsafe fn slice_mut<R>(&self, start: i32, len: i32, f: impl FnOnce(&mut [u8]) -> R) -> R {
        let store = self.store.lock().unwrap();
        mem_slice_mut(&store.as_store_ref(), &self.memory, start, len, f)
    }
}

struct HostFunction {
    store: HostStore,
    func: Function,
}

impl HostFunction {
    pub fn new(store: HostStore, func: Function) -> Self {
        Self { store, func }
    }

    pub fn call(&self, params: &[Value]) -> Result<Box<[Value]>, RuntimeError> {
        self.func
            .call(&mut self.store.lock().unwrap().as_store_mut(), params)
    }
}

/// A Wasmer [`Instance`].
pub struct WasmerModule {
    instance: HostInstance,
    memory: HostMemory,
    abi_free: HostFunction,
    abi_alloc: HostFunction,
}

impl WasmerModule {
    /// Loads the WASM [`Module`], with some imports.
    pub(crate) fn new(store: HostStore, instance: Instance) -> Result<Self> {
        let instance = HostInstance::new(store, instance);
        let memory = instance.get_memory()?;
        let abi_free = instance.get_func(ABI_FREE_NAME)?;
        let abi_alloc = instance.get_func(ABI_ALLOC_NAME)?;
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

    type Func = WasmerFunction;

    fn call<T>(&self, name: &str, data: &[u8], f: impl FnOnce(&[u8]) -> Result<T>) -> Result<T> {
        let memory = &self.memory;
        let func = self.instance.get_func(name)?;

        let ptr = self.abi_alloc.call(&[Value::I32(data.len() as i32)])?[0].unwrap_i32();
        defer! { self.abi_free.call(&[Value::I32(ptr), Value::I32(data.len() as i32)]).unwrap(); }
        unsafe { memory.slice_mut(ptr, data.len() as i32, |s| s.copy_from_slice(data)) };

        let res = func.call(&[Value::I32(data.len() as i32), Value::I32(ptr)])?[0].unwrap_i64();
        let (len, res) = ((res >> 32) as i32, (res & 0xFFFFFFFF) as i32);
        defer! { self.abi_free.call(&[Value::I32(res), Value::I32(len)]).unwrap(); }

        let res_data = unsafe { memory.slice(res, len, |s| f(s)) }?;
        Ok(res_data)
    }
}

#[doc(hidden)]
#[derive(Clone)]
pub struct RuntimeInstanceData {
    memory: Option<Memory>,
    abi_alloc: Option<TypedFunction<i32, i32>>,
    #[allow(clippy::type_complexity)]
    func: Arc<dyn (Fn(&[u8]) -> Result<Vec<u8>>) + Send + Sync + 'static>,
}

impl RuntimeInstanceData {
    pub fn new(func: impl (Fn(&[u8]) -> Result<Vec<u8>>) + Send + Sync + 'static) -> Self {
        Self {
            memory: None,
            abi_alloc: None,
            func: Arc::new(func),
        }
    }

    pub fn set_memory(&mut self, memory: Memory) {
        self.memory = Some(memory);
    }

    pub fn memory(&self) -> &Memory {
        self.memory.as_ref().unwrap()
    }

    pub fn set_abi_alloc(&mut self, func: TypedFunction<i32, i32>) {
        self.abi_alloc = Some(func);
    }

    pub fn call(mut this: FunctionEnvMut<Self>, len: i32, ptr: i32) -> u64 {
        unsafe {
            let data = mem_slice(&this, this.data().memory(), ptr, len, |slice| {
                (this.data().func)(slice).unwrap()
            });
            let abi_alloc = this.data().abi_alloc.clone().unwrap();
            let ptr = abi_alloc.call(&mut this, data.len() as i32).unwrap();
            mem_slice_mut(
                &this,
                this.data().memory(),
                ptr,
                data.len() as i32,
                |slice| {
                    slice.copy_from_slice(&data);
                },
            );
            ((data.len() as u64) << 32) | (ptr as u64)
        }
    }
}

/// A Wasmer [`Store`] with some [`ImportObject`]s.
pub struct WasmerStoreLinker {
    store: HostStore,
    wasi_env: WasiEnv,
    imports: HashMap<String, HashMap<String, WasmerFunction>>,
}

impl WasmerStoreLinker {
    fn wrap_impl(
        store: &mut impl AsStoreMut,
        func: WasmerFunction,
    ) -> (Function, Option<FunctionEnv<RuntimeInstanceData>>) {
        let env_data = func.0;
        let env = FunctionEnv::new(store, env_data);
        let func = Function::new_typed_with_env(store, &env, RuntimeInstanceData::call);
        (func, Some(env))
    }
}

impl StoreLinker<WasmerModule> for WasmerStoreLinker {
    fn new(root_path: impl AsRef<Path>) -> Result<Self> {
        let store = Store::default();
        let wasi_state = WasiState::new("ayaka-plugin-wasmer")
            .preopen(|p| p.directory(root_path.as_ref()).alias("/").read(true))?
            .build()?;
        let wasi_env = WasiEnv::new(wasi_state);
        Ok(Self {
            store: Arc::new(Mutex::new(store)),
            wasi_env,
            imports: HashMap::default(),
        })
    }

    fn create(&self, binary: &[u8]) -> Result<WasmerModule> {
        let instance = {
            let mut store = self.store.lock().unwrap();
            let module = Module::from_binary(&store.as_store_ref(), binary)?;
            let wasi_env = self.wasi_env.clone();
            let mut wasi_env = WasiFunctionEnv::new(&mut store.as_store_mut(), wasi_env);
            let mut wasi_import = generate_import_object_from_env(
                &mut store.as_store_mut(),
                &wasi_env.env,
                WasiVersion::Latest,
            );
            let mut envs = vec![];
            for (ns, funcs) in &self.imports {
                wasi_import.register_namespace(
                    ns,
                    funcs.iter().map(|(name, func)| {
                        (
                            name.clone(),
                            Extern::Function({
                                let (func, env) =
                                    Self::wrap_impl(&mut store.as_store_mut(), func.clone());
                                if let Some(env) = env {
                                    envs.push(env)
                                }
                                func
                            }),
                        )
                    }),
                );
            }
            let instance = Instance::new(&mut store.as_store_mut(), &module, &wasi_import)?;
            wasi_env.initialize(&mut store.as_store_mut(), &instance)?;
            let memory = instance.exports.get_memory(MEMORY_NAME)?;
            let abi_alloc = instance
                .exports
                .get_typed_function(&store.as_store_ref(), ABI_ALLOC_NAME)?;
            let mut store = store.as_store_mut();
            for env in &envs {
                let env_mut = env.as_mut(&mut store);
                env_mut.set_memory(memory.clone());
                env_mut.set_abi_alloc(abi_alloc.clone());
            }
            instance
        };
        let host = WasmerModule::new(self.store.clone(), instance)?;
        Ok(host)
    }

    fn import(
        &mut self,
        ns: impl Into<String>,
        funcs: HashMap<String, WasmerFunction>,
    ) -> Result<()> {
        self.imports.insert(ns.into(), funcs);
        Ok(())
    }

    fn wrap_raw(
        &self,
        f: impl (Fn(&[u8]) -> Result<Vec<u8>>) + Send + Sync + 'static,
    ) -> WasmerFunction {
        WasmerFunction(RuntimeInstanceData::new(f))
    }
}

/// Represents a wrapped wasmer function.
#[derive(Clone)]
pub struct WasmerFunction(RuntimeInstanceData);
