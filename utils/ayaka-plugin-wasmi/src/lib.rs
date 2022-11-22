//! Wasmi-based plugin backend.

#![warn(missing_docs)]

use ayaka_plugin::*;
use scopeguard::defer;
use std::{
    collections::HashMap,
    path::Path,
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

struct HostInstance {
    store: HostStore,
    instance: Instance,
}

impl HostInstance {
    pub fn new(store: HostStore, instance: Instance) -> Self {
        Self { store, instance }
    }

    pub fn get_memory(&self) -> Option<HostMemory> {
        self.instance
            .get_export(self.store.lock().unwrap().as_context_mut(), MEMORY_NAME)
            .map(|mem| HostMemory::new(self.store.clone(), mem.into_memory().unwrap()))
    }

    pub fn get_typed_func<Params: WasmParams, Results: WasmResults>(
        &self,
        name: &str,
    ) -> Result<HostTypedFunc<Params, Results>, wasmi::Error> {
        let func = {
            let mut store = self.store.lock().unwrap();
            self.instance
                .get_export(store.as_context_mut(), name)
                .unwrap()
                .into_func()
                .unwrap()
                .typed(store.as_context())?
        };
        Ok(HostTypedFunc::new(self.store.clone(), func))
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
        let data = mem_slice(store.as_context(), &self.memory, start, len);
        f(data)
    }

    pub unsafe fn slice_mut<R>(&self, start: i32, len: i32, f: impl FnOnce(&mut [u8]) -> R) -> R {
        let mut store = self.store.lock().unwrap();
        let data = mem_slice_mut(store.as_context_mut(), &self.memory, start, len);
        f(data)
    }
}

struct HostTypedFunc<Params, Results> {
    store: HostStore,
    func: TypedFunc<Params, Results>,
}

impl<Params: WasmParams, Results: WasmResults> HostTypedFunc<Params, Results> {
    pub fn new(store: HostStore, func: TypedFunc<Params, Results>) -> Self {
        Self { store, func }
    }

    pub fn call(&self, params: Params) -> Result<Results, Trap> {
        self.func
            .call(self.store.lock().unwrap().as_context_mut(), params)
    }
}

/// A Wasmi [`Instance`].
pub struct WasmiModule {
    instance: HostInstance,
    memory: HostMemory,
    abi_free: HostTypedFunc<(i32, i32), ()>,
    abi_alloc: HostTypedFunc<i32, i32>,
}

impl WasmiModule {
    pub(crate) fn new(store: HostStore, module: &Module, linker: &Linker<WasiCtx>) -> Result<Self> {
        let instance = {
            let mut store = store.lock().unwrap();
            linker
                .instantiate(store.as_context_mut(), module)?
                .start(store.as_context_mut())?
        };
        let instance = HostInstance::new(store, instance);
        let memory = instance.get_memory().unwrap();
        let abi_free = instance.get_typed_func(ABI_FREE_NAME)?;
        let abi_alloc = instance.get_typed_func(ABI_ALLOC_NAME)?;
        Ok(Self {
            instance,
            memory,
            abi_free,
            abi_alloc,
        })
    }
}

impl RawModule for WasmiModule {
    type Linker = WasmiStoreLinker;

    type Func = Func;

    fn call<T>(&self, name: &str, data: &[u8], f: impl FnOnce(&[u8]) -> Result<T>) -> Result<T> {
        let memory = &self.memory;
        let func = self.instance.get_typed_func::<(i32, i32), u64>(name)?;

        let ptr = self.abi_alloc.call(data.len() as i32)?;
        defer! { self.abi_free.call((ptr, data.len() as i32)).unwrap(); }
        unsafe { memory.slice_mut(ptr, data.len() as i32, |s| s.copy_from_slice(data)) };

        let res = func.call((data.len() as i32, ptr))?;
        let (len, res) = ((res >> 32) as i32, (res & 0xFFFFFFFF) as i32);
        defer! { self.abi_free.call((res, len)).unwrap(); }

        let res_data = unsafe { memory.slice(res, len, |s| f(s)) }?;
        Ok(res_data)
    }
}

/// A Wasmi [`Store`] with [`Linker`].
pub struct WasmiStoreLinker {
    engine: Engine,
    store: HostStore,
    linker: Linker<WasiCtx>,
}

impl WasmiStoreLinker {
    fn preopen_root(root_path: impl AsRef<Path>) -> Result<cap_std::fs::Dir> {
        let mut options = std::fs::OpenOptions::new();
        options.read(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_mode(3); // remove FILE_SHARE_DELETE
            options.custom_flags(0x02000000); // open dir with FILE_FLAG_BACKUP_SEMANTICS
        }
        let root = options.open(root_path)?;
        Ok(cap_std::fs::Dir::from_std_file(root))
    }
}

impl StoreLinker<WasmiModule> for WasmiStoreLinker {
    fn new(root_path: impl AsRef<Path>) -> Result<Self> {
        let engine = Engine::default();
        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .preopened_dir(Self::preopen_root(root_path)?, "/")?
            .build();
        let mut store = Store::new(&engine, wasi);
        let mut linker = Linker::new();
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

    fn wrap(&self, f: impl Fn() + Send + Sync + 'static) -> Func {
        Func::wrap(self.store.lock().unwrap().as_context_mut(), f)
    }

    fn wrap_with_args_raw(
        &self,
        f: impl (Fn(&[u8]) -> Result<()>) + Send + Sync + 'static,
    ) -> Func {
        Func::wrap(
            self.store.lock().unwrap().as_context_mut(),
            move |store: Caller<WasiCtx>, len: i32, data: i32| unsafe {
                let memory = store
                    .get_export(MEMORY_NAME)
                    .unwrap()
                    .into_memory()
                    .unwrap();
                let data = mem_slice(store.as_context(), &memory, data, len);
                f(data).map_err(|e| Trap::new(e.to_string()))?;
                Ok(())
            },
        )
    }
}
