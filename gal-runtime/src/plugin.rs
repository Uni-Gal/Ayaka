use crate::*;
use std::sync::Mutex;
use wit_bindgen_wasmtime::{
    anyhow,
    rt::{copy_slice, invalid_variant, RawMem},
    wasmtime::*,
};

pub struct Host {
    canonical_abi_free: TypedFunc<(i32, i32, i32), ()>,
    canonical_abi_realloc: TypedFunc<(i32, i32, i32, i32), i32>,
    instance: Instance,
    memory: Memory,
}
impl Host {
    pub fn instantiate(
        mut store: impl AsContextMut<Data = ()>,
        module: &Module,
        linker: &mut Linker<()>,
    ) -> anyhow::Result<Self> {
        linker.func_wrap(
            "log",
            "__log",
            |mut caller: Caller<'_, ()>,
             level: i32,
             target_len: i32,
             target: i32,
             msg_len: i32,
             msg: i32| {
                let memory = match caller.get_export("memory") {
                    Some(Extern::Memory(mem)) => mem,
                    _ => return Err(Trap::new("failed to find host memory")),
                };
                let target_data = copy_slice(&mut caller, &memory, target, target_len, 1)?;
                let msg_data = copy_slice(&mut caller, &memory, msg, msg_len, 1)?;
                let target =
                    String::from_utf8(target_data).map_err(|_| Trap::new("invalid utf-8"))?;
                let msg = String::from_utf8(msg_data).map_err(|_| Trap::new("invalid utf-8"))?;
                let level = unsafe { std::mem::transmute(level as usize) };
                log::logger().log(
                    &log::Record::builder()
                        .level(level)
                        .target(&target)
                        .args(format_args!("{}", msg))
                        .build(),
                );
                Ok(())
            },
        )?;
        linker.func_wrap("log", "__log_flush", || log::logger().flush())?;

        let instance = linker.instantiate(&mut store, module)?;
        Ok(Self::new(store, instance)?)
    }

    pub fn new(
        mut store: impl AsContextMut<Data = ()>,
        instance: Instance,
    ) -> anyhow::Result<Self> {
        let mut store = store.as_context_mut();
        let canonical_abi_free =
            instance.get_typed_func::<(i32, i32, i32), (), _>(&mut store, "canonical_abi_free")?;
        let canonical_abi_realloc = instance
            .get_typed_func::<(i32, i32, i32, i32), i32, _>(&mut store, "canonical_abi_realloc")?;
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| Trap::new("failed to find host memory"))?;
        Ok(Self {
            canonical_abi_free,
            canonical_abi_realloc,
            instance,
            memory,
        })
    }

    pub fn dispatch(
        &self,
        mut caller: impl AsContextMut<Data = ()>,
        name: &str,
        args: &[RawValue],
    ) -> Result<RawValue, Trap> {
        let func = self
            .instance
            .get_typed_func::<(i32, i32), (i32,), _>(&mut caller, name)?;
        let func_canonical_abi_realloc = &self.canonical_abi_realloc;
        let func_canonical_abi_free = &self.canonical_abi_free;
        let memory = &self.memory;
        let vec1 = args;
        let len1 = vec1.len() as i32;
        let result1 = func_canonical_abi_realloc.call(&mut caller, (0, 0, 8, len1 * 16))?;
        for (i, e) in vec1.into_iter().enumerate() {
            let base = result1 + (i as i32) * 16;
            {
                match e {
                    RawValue::Unit => {
                        memory
                            .data_mut(&mut caller)
                            .store(base + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                    }
                    RawValue::Bool(e) => {
                        memory
                            .data_mut(&mut caller)
                            .store(base + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                        match e {
                            false => {
                                memory.data_mut(&mut caller).store(
                                    base + 8,
                                    wit_bindgen_wasmtime::rt::as_i32(0i32) as u8,
                                )?;
                            }
                            true => {
                                memory.data_mut(&mut caller).store(
                                    base + 8,
                                    wit_bindgen_wasmtime::rt::as_i32(1i32) as u8,
                                )?;
                            }
                        };
                    }
                    RawValue::Num(e) => {
                        memory
                            .data_mut(&mut caller)
                            .store(base + 0, wit_bindgen_wasmtime::rt::as_i32(2i32) as u8)?;
                        memory.data_mut(&mut caller).store(
                            base + 8,
                            wit_bindgen_wasmtime::rt::as_i64(wit_bindgen_wasmtime::rt::as_i64(e)),
                        )?;
                    }
                    RawValue::Str(e) => {
                        memory
                            .data_mut(&mut caller)
                            .store(base + 0, wit_bindgen_wasmtime::rt::as_i32(3i32) as u8)?;
                        let vec0 = e;
                        let ptr0 = func_canonical_abi_realloc
                            .call(&mut caller, (0, 0, 1, (vec0.len() as i32) * 1))?;
                        memory
                            .data_mut(&mut caller)
                            .store_many(ptr0, vec0.as_ref())?;
                        memory.data_mut(&mut caller).store(
                            base + 12,
                            wit_bindgen_wasmtime::rt::as_i32(vec0.len() as i32),
                        )?;
                        memory
                            .data_mut(&mut caller)
                            .store(base + 8, wit_bindgen_wasmtime::rt::as_i32(ptr0))?;
                    }
                };
            }
        }
        let (result2_0,) = func.call(&mut caller, (result1, len1))?;
        let load3 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 0)?;
        let load4 = memory.data_mut(&mut caller).load::<i64>(result2_0 + 8)?;
        let load5 = memory.data_mut(&mut caller).load::<i32>(result2_0 + 16)?;
        Ok(match load3 {
            0 => RawValue::Unit,
            1 => RawValue::Bool(match load4 as i32 {
                0 => false,
                1 => true,
                _ => return Err(invalid_variant("bool")),
            }),
            2 => RawValue::Num(load4),
            3 => RawValue::Str({
                let ptr6 = load4 as i32;
                let len6 = load5;

                let data6 = copy_slice(&mut caller, memory, ptr6, len6, 1)?;
                func_canonical_abi_free.call(&mut caller, (ptr6, len6 * 1, 1))?;
                String::from_utf8(data6).map_err(|_| Trap::new("invalid utf-8"))?
            }),
            _ => return Err(invalid_variant("RawValue")),
        })
    }
}

pub struct Runtime {
    pub store: Mutex<Store<()>>,
    pub modules: HashMap<String, Host>,
}

impl Runtime {
    pub fn load(dir: impl AsRef<Path>, rel_to: impl AsRef<Path>) -> anyhow::Result<Self> {
        let mut store = Store::<()>::default();
        let mut linker = Linker::new(store.engine());
        let mut path = rel_to.as_ref().to_path_buf();
        path.push(dir);
        let mut modules = HashMap::new();
        for f in std::fs::read_dir(path)? {
            let p = f?.path();
            if p.extension()
                .map(|s| s.to_string_lossy())
                .unwrap_or_default()
                == "wasm"
            {
                let buf = std::fs::read(&p)?;
                let module = Module::new(store.engine(), buf)?;
                let runtime = Host::instantiate(&mut store, &module, &mut linker)?;
                modules.insert(
                    p.with_extension("")
                        .file_name()
                        .map(|s| s.to_string_lossy())
                        .unwrap_or_default()
                        .into_owned(),
                    runtime,
                );
            }
        }
        Ok(Self {
            store: Mutex::new(store),
            modules,
        })
    }
}
