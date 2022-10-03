//! The plugin utilities.

#![allow(unsafe_code)]
#![allow(clippy::mut_from_ref)]

use crate::*;
use anyhow::Result;
use ayaka_bindings_types::*;
use scopeguard::defer;
use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;
use stream_future::stream;
use tryiterator::TryIteratorExt;
use wasmer::*;
use wasmer_wasi::*;

/// An instance of a WASM plugin module.
pub struct Host {
    instance: Instance,
    memory: Memory,
    abi_free: NativeFunc<(i32, i32), ()>,
    abi_alloc: NativeFunc<i32, i32>,
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
    fn new(module: &Module, resolver: &(dyn Resolver + Send + Sync)) -> Result<Self> {
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

impl PluginModule for Host {
    fn call<P: Serialize, R: DeserializeOwned>(&self, name: &str, args: P) -> Result<R> {
        let memory = &self.memory;
        let func = self
            .instance
            .exports
            .get_native_function::<(i32, i32), u64>(name)?;

        let data = rmp_serde::to_vec(&args)?;

        let ptr = self.abi_alloc.call(data.len() as i32)?;
        defer! { self.abi_free.call(ptr, data.len() as i32).unwrap(); }
        unsafe { mem_slice_mut(memory, ptr, data.len() as i32) }.copy_from_slice(&data);

        let res = func.call(data.len() as i32, ptr)?;
        let (len, res) = ((res >> 32) as i32, (res & 0xFFFFFFFF) as i32);
        defer! { self.abi_free.call(res, len).unwrap(); }

        let res_data = unsafe { mem_slice(memory, res, len) };
        let res_data = rmp_serde::from_slice(res_data)?;
        Ok(res_data)
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
    pub unsafe fn import<Params: DeserializeOwned, Res: Serialize>(
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

/// The store of the WASM runtime.
pub struct HostStore {
    store: Store,
    imports: Box<dyn NamedResolver + Send + Sync>,
}

impl HostStore {
    fn new() -> Result<Self> {
        let store = Store::default();
        let imports = Self::imports(&store)?;
        Ok(Self { store, imports })
    }

    fn imports(store: &Store) -> Result<Box<dyn NamedResolver + Send + Sync>> {
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
            .preopen_dir("/")?
            .finalize()?;
        let wasi_import = generate_import_object_from_env(store, wasi_env, WasiVersion::Latest);
        Ok(Box::new(import_object.chain_front(wasi_import)))
    }

    fn from_binary(&self, binary: &[u8]) -> Result<Host> {
        let module = Module::from_binary(&self.store, binary)?;
        let host = Host::new(&module, &self.imports)?;
        Ok(host)
    }
}

/// The specific variable table.
pub type HostVarTable<'a> = VarTable<'a, Runtime>;

/// The plugin runtime.
pub struct Runtime(PluginRuntime<Host>);

/// The load status of [`Runtime`].
#[derive(Debug, Clone)]
pub enum LoadStatus {
    /// Start creating the engine.
    CreateEngine,
    /// Loading the plugin.
    LoadPlugin(String, usize, usize),
}

impl Runtime {
    /// Load plugins from specific directory and plugin names.
    ///
    /// The actual load folder will be `rel_to.join(dir)`.
    ///
    /// If `names` is empty, all WASM files will be loaded.
    #[stream(LoadStatus, lifetime = "'a")]
    pub async fn load<'a>(
        dir: impl AsRef<Path> + 'a,
        rel_to: impl AsRef<Path> + 'a,
        names: &'a [impl AsRef<str>],
    ) -> Result<Self> {
        let path = rel_to.as_ref().join(dir);
        yield LoadStatus::CreateEngine;
        let store = HostStore::new()?;
        let mut inner = PluginRuntime::new();
        let paths = if names.is_empty() {
            std::fs::read_dir(path)?
                .try_filter_map(|f| {
                    let p = f.path();
                    if p.is_file() && p.extension().unwrap_or_default() == "wasm" {
                        let name = p
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned();
                        Ok(Some((name, p)))
                    } else {
                        Ok(None)
                    }
                })
                .try_collect::<Vec<_>>()?
        } else {
            names
                .iter()
                .filter_map(|name| {
                    let name = name.as_ref();
                    let p = path.join(name).with_extension("wasm");
                    if p.exists() {
                        Some((name.to_string(), p))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };
        let total_len = paths.len();
        for (i, (name, p)) in paths.into_iter().enumerate() {
            yield LoadStatus::LoadPlugin(name.clone(), i, total_len);
            let buf = std::fs::read(p)?;
            let module = store.from_binary(&buf)?;
            inner.insert_module(name, module)?;
        }
        Ok(Self(inner))
    }
}

impl PluginContext for Runtime {
    type Module = Host;

    fn get_module(&self, name: &str) -> Option<&Self::Module> {
        self.0.get_module(name)
    }

    fn find_text_module(&self, cmd: &str) -> Option<&Self::Module> {
        self.0.find_text_module(cmd)
    }

    fn find_line_module(&self, cmd: &str) -> Option<&Self::Module> {
        self.0.find_line_module(cmd)
    }

    type ActionMIter<'a> = <PluginRuntime<Host> as PluginContext>::ActionMIter<'a>;

    fn action_modules(&self) -> Self::ActionMIter<'_> {
        self.0.action_modules()
    }

    type GameMIter<'a> = <PluginRuntime<Host> as PluginContext>::GameMIter<'a>;

    fn game_modules(&self) -> Self::GameMIter<'_> {
        self.0.game_modules()
    }
}

#[cfg(test)]
mod test {
    use crate::plugin::*;
    use tokio::sync::OnceCell;

    static RUNTIME: OnceCell<Runtime> = OnceCell::const_new();

    async fn with_ctx(f: impl FnOnce(&mut HostVarTable)) {
        let runtime = RUNTIME
            .get_or_init(|| async {
                let runtime = Runtime::load(
                    "../../examples/plugins",
                    env!("CARGO_MANIFEST_DIR"),
                    &["random"],
                );
                runtime.await.unwrap()
            })
            .await;
        let mut locals = VarMap::default();
        let mut ctx = VarTable::new(runtime, &mut locals);
        f(&mut ctx);
    }

    #[tokio::test]
    async fn vars() {
        with_ctx(|ctx| {
            assert_eq!(
                ProgramParser::new()
                    .parse(
                        "
                            a = 0;
                            a += 1;
                            a += a;
                            a
                        "
                    )
                    .ok()
                    .call(ctx),
                RawValue::Num(2)
            );

            assert_eq!(
                ProgramParser::new().parse("a").ok().call(ctx),
                RawValue::Unit
            );

            assert_eq!(
                ProgramParser::new()
                    .parse(
                        "
                            $a = 0;
                            $a += 1;
                            $a += a;
                            $a
                        "
                    )
                    .ok()
                    .call(ctx),
                RawValue::Num(1)
            );

            assert_eq!(
                ProgramParser::new().parse("$a").ok().call(ctx),
                RawValue::Num(1)
            );
        })
        .await;
    }

    #[tokio::test]
    async fn if_test() {
        with_ctx(|ctx| {
            assert_eq!(
                ProgramParser::new()
                    .parse(
                        r##"
                            if(1 + 1 + 4 + 5 + 1 + 4 == 16, "sodayo", ~)
                        "##
                    )
                    .ok()
                    .call(ctx)
                    .get_num(),
                6
            );
            assert_eq!(
                ProgramParser::new()
                    .parse(
                        r##"
                            if(true, "sodayo")
                        "##
                    )
                    .ok()
                    .call(ctx)
                    .get_str(),
                "sodayo"
            );
        })
        .await;
    }

    #[tokio::test]
    async fn random() {
        with_ctx(|ctx| {
            assert!((0..10).contains(
                &ProgramParser::new()
                    .parse(
                        r##"
                            random.rnd(10)
                        "##
                    )
                    .ok()
                    .call(ctx)
                    .get_num()
            ))
        })
        .await;
    }
}
