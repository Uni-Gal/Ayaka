use crate::{progress_future::ProgressFuture, *};
use anyhow::Result;
use gal_bindings_types::*;
use log::warn;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio_stream::{wrappers::ReadDirStream, StreamExt};
use wasmer::*;
use wasmer_wasi::*;

pub struct Host {
    abi_free: NativeFunc<(i32, i32, i32), ()>,
    abi_alloc: NativeFunc<(i32, i32), i32>,
    export_free: NativeFunc<(i32, i32), ()>,
    instance: Instance,
}

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

impl Host {
    pub fn new(module: &Module, resolver: &(dyn Resolver + Send + Sync)) -> Result<Self> {
        let instance = Instance::new(module, resolver)?;
        let abi_free = instance.exports.get_native_function("__abi_free")?;
        let abi_alloc = instance.exports.get_native_function("__abi_alloc")?;
        let export_free = instance.exports.get_native_function("__export_free")?;
        Ok(Self {
            abi_free,
            abi_alloc,
            export_free,
            instance,
        })
    }

    pub fn call<Params: Serialize, Res: DeserializeOwned>(
        &self,
        name: &str,
        args: Params,
    ) -> Result<Res> {
        let memory = self.instance.exports.get_memory("memory")?;
        let func = self
            .instance
            .exports
            .get_native_function::<(i32, i32), u64>(name)?;
        let data = rmp_serde::to_vec(&args)?;
        let ptr = self.abi_alloc.call(8, data.len() as i32)?;
        unsafe { mem_slice_mut(&memory, ptr, data.len() as i32) }.copy_from_slice(&data);
        let res = func.call(data.len() as i32, ptr)?;
        let (len, res) = ((res >> 32) as i32, (res & 0xFFFFFFFF) as i32);
        self.abi_free.call(ptr, data.len() as i32, 8)?;
        let res_data = unsafe { mem_slice(&memory, res, len) };
        let res_data = rmp_serde::from_slice(res_data)?;
        self.export_free.call(len, res)?;
        Ok(res_data)
    }

    pub fn dispatch_method(&self, name: &str, args: &[RawValue]) -> Result<RawValue> {
        self.call(name, (args,))
    }

    pub fn plugin_type(&self) -> Result<PluginType> {
        self.call("plugin_type", ())
    }

    pub fn process_action(&self, frontend: FrontendType, action: Action) -> Result<Action> {
        self.call("process_action", (frontend, action))
    }

    pub fn text_commands(&self) -> Result<Vec<String>> {
        self.call("text_commands", ())
    }

    pub fn dispatch_command(
        &self,
        name: &str,
        args: &[String],
        ctx: TextProcessContextRef,
    ) -> Result<TextProcessResult> {
        self.call(name, (args, ctx))
    }
}

pub struct Runtime {
    pub store: Store,
    pub modules: HashMap<String, Host>,
    pub action_modules: Vec<String>,
    pub text_modules: HashMap<String, String>,
}

pub struct RuntimeRef<'a> {
    pub store: &'a mut Store,
    pub modules: &'a HashMap<String, Host>,
}

#[derive(Debug, Clone)]
pub enum LoadStatus {
    CreateEngine,
    LoadPlugin(String, usize, usize),
}

#[derive(Default, Clone, WasmerEnv)]
struct RuntimeInstanceData {
    #[wasmer(export)]
    memory: LazyInit<Memory>,
}

impl Runtime {
    fn imports(store: &Store) -> Result<Box<dyn NamedResolver + Send + Sync>> {
        let log_func = Function::new_native_with_env(
            store,
            RuntimeInstanceData::default(),
            |env_data: &RuntimeInstanceData, len: i32, data: i32| {
                let memory = unsafe { env_data.memory.get_unchecked() };
                let data = unsafe { mem_slice(memory, data, len) };
                let data: Record = rmp_serde::from_slice(data).unwrap();
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
            },
        );
        let log_flush_func = Function::new_native(store, || log::logger().flush());
        let exists_func = Function::new_native_with_env(
            store,
            RuntimeInstanceData::default(),
            |env_data: &RuntimeInstanceData, len: i32, data: i32| {
                let memory = unsafe { env_data.memory.get_unchecked() };
                let data = unsafe { mem_slice(&memory, data, len) };
                let path = PathBuf::from(unsafe { std::str::from_utf8_unchecked(data) });
                path.exists() as i32
            },
        );
        let import_object = imports! {
            "log" => {
                "__log" => log_func,
                "__log_flush" => log_flush_func,
            },
            "fs" => {
                "__exists" => exists_func,
            }
        };
        let wasi_env = WasiState::new("gal-runtime").finalize()?;
        let wasi_import = generate_import_object_from_env(store, wasi_env, WasiVersion::Latest);
        Ok(Box::new(import_object.chain_front(wasi_import)))
    }

    pub fn load<'a>(
        dir: impl AsRef<Path> + 'a,
        rel_to: impl AsRef<Path> + 'a,
        names: Vec<String>,
    ) -> ProgressFuture<Result<Self>, LoadStatus> {
        let path = rel_to.as_ref().join(dir);
        ProgressFuture::new(LoadStatus::CreateEngine, async move |tx| {
            let store = Store::default();
            let import_object = Self::imports(&store)?;
            let mut modules = HashMap::new();
            let mut action_modules = Vec::new();
            let mut text_modules = HashMap::new();
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
                            .file_stem()
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
                let module = Module::from_binary(&store, &buf)?;
                let runtime = Host::new(&module, &import_object)?;
                match runtime.plugin_type()? {
                    PluginType::Script => {}
                    PluginType::Action => {
                        action_modules.push(name.clone());
                    }
                    PluginType::Text => {
                        let cmds = runtime.text_commands()?;
                        for cmd in cmds.into_iter() {
                            let res = text_modules.insert(cmd.clone(), name.clone());
                            if let Some(old_module) = res {
                                warn!(
                                    "Command `{}` is overrided by \"{}\" over \"{}\"",
                                    cmd, name, old_module
                                );
                            }
                        }
                    }
                }
                modules.insert(name, runtime);
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
