use crate::*;
use wit_bindgen_wasmtime::{
    anyhow,
    rt::{copy_slice, invalid_variant, RawMem},
    wasmtime::*,
};

pub struct Input {
    canonical_abi_free: TypedFunc<(i32, i32, i32), ()>,
    canonical_abi_realloc: TypedFunc<(i32, i32, i32, i32), i32>,
    dispatch: TypedFunc<(i32, i32, i32, i32), (i32,)>,
    memory: Memory,
}
impl Input {
    pub fn instantiate(
        mut store: impl AsContextMut<Data = ()>,
        module: &Module,
        linker: &mut Linker<()>,
    ) -> anyhow::Result<Self> {
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
        let dispatch =
            instance.get_typed_func::<(i32, i32, i32, i32), (i32,), _>(&mut store, "dispatch")?;
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow::anyhow!("`memory` export not a memory"))?;
        Ok(Input {
            canonical_abi_free,
            canonical_abi_realloc,
            dispatch,
            memory,
        })
    }
    pub fn dispatch(
        &self,
        mut caller: impl AsContextMut<Data = ()>,
        name: &str,
        args: &[RawValue],
    ) -> Result<Option<RawValue>, Trap> {
        let func_canonical_abi_free = &self.canonical_abi_free;
        let func_canonical_abi_realloc = &self.canonical_abi_realloc;
        let memory = &self.memory;
        let vec0 = name;
        let ptr0 =
            func_canonical_abi_realloc.call(&mut caller, (0, 0, 1, (vec0.len() as i32) * 1))?;
        memory
            .data_mut(&mut caller)
            .store_many(ptr0, vec0.as_ref())?;
        let vec2 = args;
        let len2 = vec2.len() as i32;
        let result2 = func_canonical_abi_realloc.call(&mut caller, (0, 0, 8, len2 * 16))?;
        for (i, e) in vec2.into_iter().enumerate() {
            let base = result2 + (i as i32) * 16;
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
                        let vec1 = e;
                        let ptr1 = func_canonical_abi_realloc
                            .call(&mut caller, (0, 0, 1, (vec1.len() as i32) * 1))?;
                        memory
                            .data_mut(&mut caller)
                            .store_many(ptr1, vec1.as_ref())?;
                        memory.data_mut(&mut caller).store(
                            base + 12,
                            wit_bindgen_wasmtime::rt::as_i32(vec1.len() as i32),
                        )?;
                        memory
                            .data_mut(&mut caller)
                            .store(base + 8, wit_bindgen_wasmtime::rt::as_i32(ptr1))?;
                    }
                };
            }
        }
        let (result3_0,) = self
            .dispatch
            .call(&mut caller, (ptr0, vec0.len() as i32, result2, len2))?;
        let load4 = memory.data_mut(&mut caller).load::<i32>(result3_0 + 0)?;
        let load5 = memory.data_mut(&mut caller).load::<i32>(result3_0 + 8)?;
        let load6 = memory.data_mut(&mut caller).load::<i64>(result3_0 + 16)?;
        let load7 = memory.data_mut(&mut caller).load::<i32>(result3_0 + 24)?;
        Ok(match load4 {
            0 => None,
            1 => Some(match load5 {
                0 => RawValue::Unit,
                1 => RawValue::Bool(match load6 as i32 {
                    0 => false,
                    1 => true,
                    _ => return Err(invalid_variant("bool")),
                }),
                2 => RawValue::Num(load6),
                3 => RawValue::Str({
                    let ptr8 = load6 as i32;
                    let len8 = load7;

                    let data8 = copy_slice(&mut caller, memory, ptr8, len8, 1)?;
                    func_canonical_abi_free.call(&mut caller, (ptr8, len8 * 1, 1))?;
                    String::from_utf8(data8).map_err(|_| Trap::new("invalid utf-8"))?
                }),
                _ => return Err(invalid_variant("RawValue")),
            }),
            _ => return Err(invalid_variant("Option")),
        })
    }
}

pub fn load_plugins(dir: impl AsRef<Path>, rel_to: impl AsRef<Path>) -> Runtime {
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let mut linker = Linker::new(&engine);
    let mut path = rel_to.as_ref().to_path_buf();
    path.push(dir);
    let modules = std::fs::read_dir(path)
        .unwrap()
        .map(|f| f.unwrap().path())
        .filter(|p| {
            p.extension()
                .map(|s| s.to_string_lossy())
                .unwrap_or_default()
                == "wasm"
        })
        .map(|p| {
            let buf = std::fs::read(&p).unwrap();
            let module = Module::new(&engine, buf).unwrap();
            let runtime = Input::instantiate(&mut store, &module, &mut linker).unwrap();
            (
                p.with_extension("")
                    .file_name()
                    .map(|s| s.to_string_lossy())
                    .unwrap_or_default()
                    .into_owned(),
                runtime,
            )
        })
        .collect();
    Runtime { store, modules }
}
