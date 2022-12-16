//! Wasmi-based plugin backend.

#![warn(missing_docs)]

use ayaka_plugin::*;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use wasmi::{core::Trap, *};
use wasmi_wasi::*;

unsafe fn mem_slice<'a, T: 'a>(
    store: impl Into<StoreContext<'a, T>>,
    memory: &Memory,
    start: i32,
    len: i32,
) -> &'a [u8] {
    memory
        .data(store)
        .get_unchecked(start as usize..)
        .get_unchecked(..len as usize)
}

unsafe fn mem_slice_mut<'a, T: 'a>(
    store: impl Into<StoreContextMut<'a, T>>,
    memory: &Memory,
    start: i32,
    len: i32,
) -> &'a mut [u8] {
    memory
        .data_mut(store)
        .get_unchecked_mut(start as usize..)
        .get_unchecked_mut(..len as usize)
}

type HostStore = Arc<Mutex<Store<WasiCtx>>>;

/// A Wasmi [`Instance`].
pub struct WasmiModule {
    store: HostStore,
    instance: Instance,
    memory: Memory,
    abi_free: TypedFunc<(i32, i32), ()>,
    abi_alloc: TypedFunc<i32, i32>,
}

impl WasmiModule {
    fn new(store: HostStore, module: &Module, linker: &wasmi::Linker<WasiCtx>) -> Result<Self> {
        let mut inner_store = store.lock().unwrap();
        let instance = linker
            .instantiate(inner_store.as_context_mut(), module)?
            .start(inner_store.as_context_mut())?;
        let memory = instance
            .get_export(inner_store.as_context(), MEMORY_NAME)
            .unwrap()
            .into_memory()
            .unwrap();
        let abi_free = instance
            .get_export(inner_store.as_context(), ABI_FREE_NAME)
            .unwrap()
            .into_func()
            .unwrap()
            .typed(inner_store.as_context())?;
        let abi_alloc = instance
            .get_export(inner_store.as_context_mut(), ABI_ALLOC_NAME)
            .unwrap()
            .into_func()
            .unwrap()
            .typed(inner_store.as_context())?;
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
        mut store: StoreContextMut<WasiCtx>,
        name: &str,
        data: &[u8],
        f: impl FnOnce(&[u8]) -> Result<T>,
    ) -> Result<T> {
        let func = self
            .instance
            .get_export(&store, name)
            .unwrap()
            .into_func()
            .unwrap()
            .typed::<(i32, i32), u64>(&store)?;

        let ptr = self.abi_alloc.call(&mut store, data.len() as i32)?;
        unsafe {
            mem_slice_mut(&mut store, &self.memory, ptr, data.len() as i32).copy_from_slice(data)
        };

        let res = func.call(&mut store, (data.len() as i32, ptr));

        self.abi_free.call(&mut store, (ptr, data.len() as i32))?;

        let res = res?;
        let (len, res) = ((res >> 32) as i32, (res & 0xFFFFFFFF) as i32);

        let res_data = unsafe { mem_slice(&store, &self.memory, res, len) };

        let res_data = f(res_data);

        self.abi_free.call(&mut store, (res, len))?;

        let res_data = res_data?;
        Ok(res_data)
    }
}

impl RawModule for WasmiModule {
    type Linker = WasmiLinker;

    type LinkerHandle<'a> = WasmiLinkerHandle<'a>;

    type Func = Func;

    fn call<T>(&self, name: &str, data: &[u8], f: impl FnOnce(&[u8]) -> Result<T>) -> Result<T> {
        self.call_impl(self.store.lock().unwrap().as_context_mut(), name, data, f)
    }
}

/// A Wasmi [`Store`] with [`Linker`].
pub struct WasmiLinker {
    engine: Engine,
    store: HostStore,
    linker: wasmi::Linker<WasiCtx>,
}

impl ayaka_plugin::Linker<WasmiModule> for WasmiLinker {
    fn new() -> Result<Self> {
        let engine = Engine::default();
        let wasi = WasiCtxBuilder::new().inherit_stdio().build();
        let mut store = Store::new(&engine, wasi);
        let mut linker = wasmi::Linker::new();
        define_wasi(&mut linker, &mut store, |ctx| ctx)?;
        Ok(Self {
            engine,
            store: Arc::new(Mutex::new(store)),
            linker,
        })
    }

    fn create(&self, binary: &[u8]) -> Result<WasmiModule> {
        let module = Module::new(&self.engine, binary)?;
        let host = WasmiModule::new(self.store.clone(), &module, &self.linker)?;
        Ok(host)
    }

    fn import(&mut self, ns: impl Into<String>, funcs: HashMap<String, Func>) -> Result<()> {
        let ns = ns.into();
        for (name, func) in funcs {
            self.linker.define(&ns, &name, func)?;
        }
        Ok(())
    }

    fn wrap_raw(
        &self,
        f: impl (Fn(WasmiLinkerHandle, i32, i32) -> Result<Vec<u8>>) + Send + Sync + 'static,
    ) -> Func {
        Func::wrap(
            self.store.lock().unwrap().as_context_mut(),
            move |mut store: Caller<WasiCtx>, len: i32, data: i32| unsafe {
                let memory = store
                    .get_export(MEMORY_NAME)
                    .unwrap()
                    .into_memory()
                    .unwrap();
                let data = {
                    let store = store.as_context_mut();
                    let handle = WasmiLinkerHandle { store, memory };
                    f(handle, data, len).map_err(|e| Trap::new(e.to_string()))?
                };
                let abi_alloc = store
                    .get_export(ABI_ALLOC_NAME)
                    .unwrap()
                    .into_func()
                    .unwrap()
                    .typed::<i32, i32>(store.as_context())
                    .map_err(|e| Trap::new(e.to_string()))?;
                let ptr = abi_alloc.call(store.as_context_mut(), data.len() as i32)?;
                mem_slice_mut(store.as_context_mut(), &memory, ptr, data.len() as i32)
                    .copy_from_slice(&data);
                Ok(((data.len() as u64) << 32) | (ptr as u64))
            },
        )
    }
}

/// A Wasmi [`StoreContextMut`].
pub struct WasmiLinkerHandle<'a> {
    store: StoreContextMut<'a, WasiCtx>,
    memory: Memory,
}

impl<'a> LinkerHandle<'a, WasmiModule> for WasmiLinkerHandle<'a> {
    fn call<T>(
        &mut self,
        m: &WasmiModule,
        name: &str,
        data: &[u8],
        f: impl FnOnce(&[u8]) -> Result<T>,
    ) -> Result<T> {
        m.call_impl(self.store.as_context_mut(), name, data, f)
    }

    fn slice<T>(&self, start: i32, len: i32, f: impl FnOnce(&[u8]) -> T) -> T {
        f(unsafe { mem_slice(self.store.as_context(), &self.memory, start, len) })
    }

    fn slice_mut<T>(&mut self, start: i32, len: i32, f: impl FnOnce(&mut [u8]) -> T) -> T {
        f(unsafe { mem_slice_mut(self.store.as_context_mut(), &self.memory, start, len) })
    }
}
