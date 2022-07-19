use crate::{progress_future::ProgressFuture, *};
use anyhow::{anyhow, Result};
use gal_bindings_types::*;
use gal_script::log::info;
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashMap, path::Path};
use tokio_stream::{wrappers::ReadDirStream, StreamExt};
use wasmtime::*;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

pub struct Host {
    abi_free: TypedFunc<(i32, i32, i32), ()>,
    abi_alloc: TypedFunc<(i32, i32), i32>,
    export_free: TypedFunc<(i32, i32), ()>,
    instance: Instance,
    memory: Memory,
}

unsafe fn mem_slice<'a>(
    memory: &Memory,
    caller: &'a mut impl AsContextMut,
    start: i32,
    len: i32,
) -> &'a [u8] {
    memory
        .data(caller)
        .get_unchecked(start as usize..)
        .get_unchecked(..len as usize)
}

unsafe fn mem_slice_mut<'a>(
    memory: &Memory,
    caller: &'a mut impl AsContextMut,
    start: i32,
    len: i32,
) -> &'a mut [u8] {
    memory
        .data_mut(caller)
        .get_unchecked_mut(start as usize..)
        .get_unchecked_mut(..len as usize)
}

impl Host {
    pub fn instantiate(
        mut store: impl AsContextMut<Data = WasiCtx>,
        module: &Module,
        linker: &mut Linker<WasiCtx>,
    ) -> Result<Self> {
        let instance = linker.instantiate(&mut store, module)?;
        Ok(Self::new(store, instance)?)
    }

    pub fn new(mut store: impl AsContextMut<Data = WasiCtx>, instance: Instance) -> Result<Self> {
        let mut store = store.as_context_mut();
        let abi_free = instance.get_typed_func(&mut store, "__abi_free")?;
        let abi_alloc = instance.get_typed_func(&mut store, "__abi_alloc")?;
        let export_free = instance.get_typed_func(&mut store, "__export_free")?;
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow!("failed to find host memory"))?;
        Ok(Self {
            abi_free,
            abi_alloc,
            export_free,
            instance,
            memory,
        })
    }

    pub fn call<Params: Serialize, Res: DeserializeOwned>(
        &self,
        mut caller: impl AsContextMut<Data = WasiCtx>,
        name: &str,
        args: Params,
    ) -> Result<Res> {
        let func = self
            .instance
            .get_typed_func::<(i32, i32), (u64,), _>(&mut caller, name)?;
        let data = rmp_serde::to_vec(&args)?;
        let ptr = self.abi_alloc.call(&mut caller, (8, data.len() as i32))?;
        unsafe { mem_slice_mut(&self.memory, &mut caller, ptr, data.len() as i32) }
            .copy_from_slice(&data);
        let (res,) = func.call(&mut caller, (data.len() as i32, ptr))?;
        let (len, res) = ((res >> 32) as i32, (res & 0xFFFFFFFF) as i32);
        self.abi_free
            .call(&mut caller, (ptr, data.len() as i32, 8))?;
        let res_data = unsafe { mem_slice(&self.memory, &mut caller, res, len) };
        let res_data = rmp_serde::from_slice(res_data)?;
        self.export_free.call(&mut caller, (len, res))?;
        Ok(res_data)
    }

    pub fn dispatch(
        &self,
        caller: impl AsContextMut<Data = WasiCtx>,
        name: &str,
        args: &[RawValue],
    ) -> Result<RawValue> {
        self.call(caller, name, (args,))
    }

    pub fn plugin_type(&self, caller: impl AsContextMut<Data = WasiCtx>) -> Result<PluginType> {
        self.call(caller, "plugin_type", ())
    }

    pub fn process_action(
        &self,
        caller: impl AsContextMut<Data = WasiCtx>,
        frontend: FrontendType,
        action: Action,
    ) -> Result<Action> {
        self.call(caller, "process_action", (frontend, action))
    }

    pub fn text_commands(&self, caller: impl AsContextMut<Data = WasiCtx>) -> Result<Vec<String>> {
        self.call(caller, "text_commands", ())
    }

    pub fn dispatch_command(
        &self,
        caller: impl AsContextMut<Data = WasiCtx>,
        name: &str,
        args: &[String],
        ctx: &TextProcessContext,
    ) -> Result<TextProcessResult> {
        self.call(caller, name, (args, ctx))
    }
}

pub struct Runtime {
    pub store: Store<WasiCtx>,
    pub modules: HashMap<String, Host>,
    pub action_modules: Vec<(String, Host)>,
    pub text_modules: HashMap<String, String>,
}

pub struct RuntimeRef<'a> {
    pub store: &'a mut Store<WasiCtx>,
    pub modules: &'a HashMap<String, Host>,
}

#[derive(Debug, Clone)]
pub enum LoadStatus {
    CreateEngine,
    LoadPlugin(String, usize, usize),
}

impl Runtime {
    fn new_linker(engine: &Engine) -> Result<Linker<WasiCtx>> {
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
                let data = unsafe { mem_slice(&memory, &mut caller, data, len) };
                let data: Record =
                    rmp_serde::from_slice(data).map_err(|e| Trap::new(e.to_string()))?;
                log::logger().log(
                    &log::Record::builder()
                        .level(data.level)
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

    pub fn load<'a>(
        dir: impl AsRef<Path> + 'a,
        rel_to: impl AsRef<Path> + 'a,
        names: Vec<String>,
    ) -> ProgressFuture<Result<Self>, LoadStatus> {
        let path = rel_to.as_ref().join(dir);
        ProgressFuture::new(LoadStatus::CreateEngine, async move |tx| {
            let wasi = WasiCtxBuilder::new().inherit_env()?.inherit_stdio().build();
            let engine = Engine::default();
            let mut store = Store::new(&engine, wasi);
            let mut modules = HashMap::new();
            let mut action_modules = Vec::new();
            let mut text_modules = HashMap::new();
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
                    let p = path.join(&name).with_extension("wasm");
                    if p.exists() {
                        paths.push((name.clone(), p));
                    }
                }
            }
            let total_len = paths.len();
            for (i, (name, p)) in paths.into_iter().enumerate() {
                tx.send(LoadStatus::LoadPlugin(name.clone(), i, total_len))?;
                let buf = tokio::fs::read(&p).await?;
                let module = Module::from_binary(store.engine(), &buf)?;
                let runtime = Host::instantiate(&mut store, &module, &mut linker)?;
                match runtime.plugin_type(&mut store)? {
                    PluginType::Script => {
                        modules.insert(name, runtime);
                    }
                    PluginType::Action => {
                        action_modules.push((name, runtime));
                    }
                    PluginType::Text => {
                        let cmds = runtime.text_commands(&mut store)?;
                        info!("Plugin {} registered: {:?}", name, &cmds);
                        for cmd in cmds.into_iter() {
                            text_modules.insert(cmd, name.clone());
                        }
                        modules.insert(name, runtime);
                    }
                }
            }
            Ok(Self {
                store,
                modules,
                action_modules,
                text_modules,
            })
        })
    }

    pub fn as_mut(&mut self) -> RuntimeRef {
        RuntimeRef {
            store: &mut self.store,
            modules: &self.modules,
        }
    }
}
