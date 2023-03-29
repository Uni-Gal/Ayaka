//! Wasmer-based plugin backend.

#![warn(missing_docs)]

use ayaka_plugin::*;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use wasmer::*;

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

/// A Wasmer [`Instance`].
pub struct WasmerModule {
    store: HostStore,
    instance: Instance,
    memory: Memory,
    abi_free: TypedFunction<(i32, i32), ()>,
    abi_alloc: TypedFunction<i32, i32>,
}

impl WasmerModule {
    /// Loads the WASM [`Module`], with some imports.
    fn new(store: HostStore, instance: Instance) -> Result<Self> {
        let memory = instance.exports.get_memory(MEMORY_NAME)?.clone();
        let inner_store = store.lock().unwrap();
        let abi_free = instance
            .exports
            .get_typed_function(&inner_store.as_store_ref(), ABI_FREE_NAME)?;
        let abi_alloc = instance
            .exports
            .get_typed_function(&inner_store.as_store_ref(), ABI_ALLOC_NAME)?;
        drop(inner_store);
        Ok(Self {
            store,
            instance,
            memory,
            abi_free,
            abi_alloc,
        })
    }

    fn call_impl<T>(
        &self,
        mut store: StoreMut,
        name: &str,
        data: &[u8],
        f: impl FnOnce(&[u8]) -> Result<T>,
    ) -> Result<T> {
        let func = self
            .instance
            .exports
            .get_typed_function::<(i32, i32), u64>(&store, name)?;

        let ptr = self.abi_alloc.call(&mut store, data.len() as i32)?;
        unsafe {
            mem_slice_mut(&store, &self.memory, ptr, data.len() as i32, |s| {
                s.copy_from_slice(data)
            })
        };

        let res = func.call(&mut store, data.len() as i32, ptr);

        self.abi_free.call(&mut store, ptr, data.len() as i32)?;

        let res = res?;
        let (len, res) = ((res >> 32) as i32, (res & 0xFFFFFFFF) as i32);

        let res_data = unsafe { mem_slice(&store, &self.memory, res, len, |s| f(s)) };

        self.abi_free.call(&mut store, res, len)?;

        let res_data = res_data?;
        Ok(res_data)
    }
}

impl RawModule for WasmerModule {
    type Linker = WasmerLinker;

    type LinkerHandle<'a> = WasmerLinkerHandle<'a>;

    type Func = WasmerFunction;

    fn call<T>(&self, name: &str, data: &[u8], f: impl FnOnce(&[u8]) -> Result<T>) -> Result<T> {
        self.call_impl(self.store.lock().unwrap().as_store_mut(), name, data, f)
    }
}

#[doc(hidden)]
#[derive(Clone)]
pub struct RuntimeInstanceData {
    memory: Option<Memory>,
    abi_alloc: Option<TypedFunction<i32, i32>>,
    #[allow(clippy::type_complexity)]
    func: Arc<dyn (Fn(WasmerLinkerHandle, i32, i32) -> Result<Vec<u8>>) + Send + Sync + 'static>,
}

impl RuntimeInstanceData {
    pub fn new(
        func: impl (Fn(WasmerLinkerHandle, i32, i32) -> Result<Vec<u8>>) + Send + Sync + 'static,
    ) -> Self {
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
        self.memory.as_ref().expect("memory should be set first")
    }

    pub fn set_abi_alloc(&mut self, func: TypedFunction<i32, i32>) {
        self.abi_alloc = Some(func);
    }

    pub fn call(mut this: FunctionEnvMut<Self>, len: i32, ptr: i32) -> u64 {
        // TODO: should we unwrap here?
        unsafe {
            let memory = this.data().memory().clone();
            let data = {
                let func = this.data().func.clone();
                let handle = WasmerLinkerHandle {
                    store: this.as_store_mut(),
                    memory: memory.clone(),
                };
                (func)(handle, ptr, len).unwrap()
            };
            let abi_alloc = this.data().abi_alloc.clone().unwrap();
            let ptr = abi_alloc.call(&mut this, data.len() as i32).unwrap();
            mem_slice_mut(&this, &memory, ptr, data.len() as i32, |slice| {
                slice.copy_from_slice(&data);
            });
            ((data.len() as u64) << 32) | (ptr as u64)
        }
    }
}

/// A Wasmer [`Store`] with some imports.
pub struct WasmerLinker {
    store: HostStore,
    imports: HashMap<String, HashMap<String, WasmerFunction>>,
}

impl WasmerLinker {
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

impl Linker<WasmerModule> for WasmerLinker {
    type Config = ();

    fn new(_: ()) -> Result<Self> {
        let store = Store::default();
        Ok(Self {
            store: Arc::new(Mutex::new(store)),
            imports: HashMap::default(),
        })
    }

    fn create(&self, binary: &[u8]) -> Result<WasmerModule> {
        let instance = {
            let mut store = self.store.lock().unwrap();
            let module = Module::from_binary(&store.as_store_ref(), binary)?;
            let mut imports = Imports::default();
            let mut envs = vec![];
            for (ns, funcs) in &self.imports {
                imports.register_namespace(
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
            let instance = Instance::new(&mut store.as_store_mut(), &module, &imports)?;
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
        f: impl (Fn(WasmerLinkerHandle, i32, i32) -> Result<Vec<u8>>) + Send + Sync + 'static,
    ) -> WasmerFunction {
        WasmerFunction(RuntimeInstanceData::new(f))
    }
}

/// Represents a wrapped wasmer function.
#[derive(Clone)]
pub struct WasmerFunction(RuntimeInstanceData);

/// A Wasmer [`StoreMut`].
pub struct WasmerLinkerHandle<'a> {
    store: StoreMut<'a>,
    memory: Memory,
}

impl<'a> LinkerHandle<'a, WasmerModule> for WasmerLinkerHandle<'a> {
    fn call<T>(
        &mut self,
        m: &WasmerModule,
        name: &str,
        data: &[u8],
        f: impl FnOnce(&[u8]) -> Result<T>,
    ) -> Result<T> {
        m.call_impl(self.store.as_store_mut(), name, data, f)
    }

    fn slice<T>(&self, start: i32, len: i32, f: impl FnOnce(&[u8]) -> T) -> T {
        unsafe { mem_slice(&self.store.as_store_ref(), &self.memory, start, len, f) }
    }

    fn slice_mut<T>(&mut self, start: i32, len: i32, f: impl FnOnce(&mut [u8]) -> T) -> T {
        unsafe { mem_slice_mut(&self.store.as_store_ref(), &self.memory, start, len, f) }
    }
}
