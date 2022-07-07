use crate::*;
use gal_primitive::{bincode, Record};
use std::{collections::HashMap, path::Path};
use tokio_stream::{wrappers::ReadDirStream, Stream, StreamExt};
use wasmtime::*;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

pub struct Host {
    canonical_abi_free: TypedFunc<(i32, i32, i32), ()>,
    canonical_abi_realloc: TypedFunc<(i32, i32, i32, i32), i32>,
    instance: Instance,
    memory: Memory,
}

impl Host {
    pub fn instantiate(
        mut store: impl AsContextMut<Data = WasiCtx>,
        module: &Module,
        linker: &mut Linker<WasiCtx>,
    ) -> anyhow::Result<Self> {
        let instance = linker.instantiate(&mut store, module)?;
        Ok(Self::new(store, instance)?)
    }

    pub fn new(
        mut store: impl AsContextMut<Data = WasiCtx>,
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
        mut caller: impl AsContextMut<Data = WasiCtx>,
        name: &str,
        args: Vec<RawValue>,
    ) -> anyhow::Result<RawValue> {
        let func = self
            .instance
            .get_typed_func::<(i32, i32), (u64,), _>(&mut caller, name)?;
        let func_canonical_abi_realloc = &self.canonical_abi_realloc;
        let func_canonical_abi_free = &self.canonical_abi_free;
        let memory = &self.memory;
        let data = bincode::encode_to_vec(args, bincode::config::standard())?;
        let ptr = func_canonical_abi_realloc.call(&mut caller, (0, 0, 8, data.len() as i32))?;
        memory
            .data_mut(&mut caller)
            .get_mut(ptr as usize..)
            .and_then(|s| s.get_mut(..data.len()))
            .map(|s| s.copy_from_slice(&data))
            .ok_or_else(|| Trap::new("out of bounds write"))?;
        let (res,) = func.call(&mut caller, (data.len() as i32, ptr))?;
        let (len, res) = ((res >> 32) as i32, (res & 0xFFFFFFFF) as i32);
        func_canonical_abi_free.call(&mut caller, (ptr, data.len() as i32, 8))?;
        let res = memory
            .data(&mut caller)
            .get(res as usize..)
            .and_then(|s| s.get(..len as usize))
            .ok_or_else(|| Trap::new("out of bounds read"))?;
        Ok(bincode::decode_from_slice(res, bincode::config::standard())?.0)
    }
}

pub struct Runtime {
    pub store: Store<WasiCtx>,
    pub modules: HashMap<String, Host>,
}

pub struct RuntimeRef<'a> {
    pub store: &'a mut Store<WasiCtx>,
    pub modules: &'a HashMap<String, Host>,
}

pub enum LoadStatus {
    LoadPlugin(String, usize, usize),
    Loaded(Runtime),
}

impl Runtime {
    fn new_linker(engine: &Engine) -> anyhow::Result<Linker<WasiCtx>> {
        let mut linker = Linker::new(engine);
        wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;
        linker.func_wrap(
            "log",
            "__log",
            |mut caller: Caller<'_, WasiCtx>, len: i32, data: i32| {
                let memory = match caller.get_export("memory") {
                    Some(Extern::Memory(mem)) => mem,
                    _ => return Err(Trap::new("failed to find host memory")),
                };
                let data = memory
                    .data(&mut caller)
                    .get(data as usize..)
                    .and_then(|s| s.get(..len as usize))
                    .ok_or_else(|| Trap::new("out of bounds read"))?;
                let data: Record = bincode::decode_from_slice(data, bincode::config::standard())
                    .map_err(|e| Trap::new(e.to_string()))?
                    .0;
                log::logger().log(
                    &log::Record::builder()
                        .level(unsafe { std::mem::transmute(data.level) })
                        .target(&data.target)
                        .args(format_args!("{}", data.msg))
                        .module_path(data.module_path.as_ref().map(|s| s.as_str()))
                        .file(data.file.as_ref().map(|s| s.as_str()))
                        .line(data.line)
                        .build(),
                );
                Ok(())
            },
        )?;
        linker.func_wrap("log", "__log_flush", || log::logger().flush())?;
        Ok(linker)
    }

    pub fn load(
        dir: impl AsRef<Path>,
        rel_to: impl AsRef<Path>,
        names: &[String],
    ) -> impl Stream<Item = anyhow::Result<LoadStatus>> + '_ {
        let path = rel_to.as_ref().join(dir);
        async_stream::try_stream! {
            let wasi = WasiCtxBuilder::new().inherit_env()?.inherit_stdio().build();
            let engine = Engine::default();
            let mut store = Store::new(&engine, wasi);
            let mut modules = HashMap::new();
            let mut linker = Self::new_linker(store.engine())?;
            let mut paths = vec![];
            if names.is_empty() {
                let mut dirs = ReadDirStream::new(tokio::fs::read_dir(path).await?);
                while let Some(f) = dirs.try_next().await? {
                    let p = f.path();
                    if p.extension()
                        .map(|s| s.to_string_lossy())
                        .unwrap_or_default()
                        == "wasm"
                    {
                        let name = p
                            .with_extension("")
                            .file_name()
                            .map(|s| s.to_string_lossy())
                            .unwrap_or_default()
                            .into_owned();
                        paths.push((name, p));
                    }
                }
            } else {
                for name in names {
                    let p = path.join(name).with_extension("wasm");
                    if p.exists() {
                        paths.push((name.clone(), p));
                    }
                }
            }
            let total_len = paths.len();
            for (i, (name, p)) in paths.into_iter().enumerate() {
                yield LoadStatus::LoadPlugin(name.clone(), i, total_len);
                let buf = tokio::fs::read(&p).await?;
                let module = Module::from_binary(store.engine(), &buf)?;
                let runtime = Host::instantiate(&mut store, &module, &mut linker)?;
                modules.insert(name, runtime);
            }
            yield LoadStatus::Loaded(Self { store, modules })
        }
    }

    pub fn as_mut(&mut self) -> RuntimeRef {
        RuntimeRef {
            store: &mut self.store,
            modules: &self.modules,
        }
    }
}
