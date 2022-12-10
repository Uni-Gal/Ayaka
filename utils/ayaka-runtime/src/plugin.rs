//! The plugin utilities.

use crate::*;
use anyhow::{bail, Result};
use ayaka_plugin::*;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, Weak},
};
use stream_future::stream;
use trylog::macros::*;
use vfs::*;

/// The plugin module with high-level interfaces.
pub struct Module<M: RawModule = BackendModule> {
    module: PluginModule<M>,
}

impl<M: RawModule> Module<M> {
    fn new(module: M) -> Self {
        Self {
            module: PluginModule::new(module),
        }
    }

    /// Gets the [`PluginType`].
    pub fn plugin_type(&self) -> Result<PluginType> {
        self.module.call("plugin_type", ())
    }

    /// Processes [`Action`] in action plugin.
    pub fn process_action(&self, ctx: ActionProcessContextRef) -> Result<ActionProcessResult> {
        self.module.call("process_action", (ctx,))
    }

    /// Calls a custom command in the text plugin.
    pub fn dispatch_text(
        &self,
        name: &str,
        args: &[String],
        ctx: TextProcessContextRef,
    ) -> Result<TextProcessResult> {
        self.module.call(name, (args, ctx))
    }

    /// Calls a custom command in the line plugin.
    pub fn dispatch_line(
        &self,
        name: &str,
        ctx: LineProcessContextRef,
    ) -> Result<LineProcessResult> {
        self.module.call(name, (ctx,))
    }

    /// Processes [`Game`] when opening the config file.
    pub fn process_game(&self, ctx: GameProcessContextRef) -> Result<GameProcessResult> {
        self.module.call("process_game", (ctx,))
    }
}

/// The plugin runtime.
pub struct Runtime<M: RawModule + Send + Sync + 'static = BackendModule> {
    modules: HashMap<String, Module<M>>,
    action_modules: Vec<String>,
    text_modules: HashMap<String, String>,
    line_modules: HashMap<String, String>,
    game_modules: Vec<String>,
}

/// The load status of [`Runtime`].
#[derive(Debug, Clone)]
pub enum LoadStatus {
    /// Start creating the engine.
    CreateEngine,
    /// Loading the plugin.
    LoadPlugin(String, usize, usize),
}

impl<M: RawModule + Send + Sync + 'static> Runtime<M> {
    fn new_linker(root_path: &VfsPath, handle: Arc<RwLock<Weak<Self>>>) -> Result<M::Linker> {
        let mut store = M::Linker::new()?;

        // log
        let log_func = store.wrap(|(data,): (Record,)| {
            let target = format!("{}::<plugin>::{}", module_path!(), data.target);
            log::logger().log(
                &log::Record::builder()
                    .level(data.level)
                    .target(&target)
                    .args(format_args!("{}", data.msg))
                    .module_path(data.module_path.as_deref())
                    .file(data.file.as_deref())
                    .line(data.line)
                    .build(),
            );
            Ok(())
        });
        let log_flush_func = store.wrap(|_: ()| {
            log::logger().flush();
            Ok(())
        });
        store.import(
            "log",
            HashMap::from([
                ("__log".to_string(), log_func),
                ("__log_flush".to_string(), log_flush_func),
            ]),
        )?;

        // plugin
        let h = handle.clone();
        let modules_func = store.wrap(move |_: ()| {
            if let Some(this) = h.read().unwrap().upgrade() {
                Ok(this.modules.keys().cloned().collect::<Vec<_>>())
            } else {
                bail!("Runtime hasn't been initialized.")
            }
        });
        let h = handle;
        let call_func = store.wrap_with(
            move |mut handle, (module, name, args): (String, String, Vec<u8>)| {
                if let Some(this) = h.read().unwrap().upgrade() {
                    Ok(handle.call(
                        this.modules[&module].module.inner(),
                        &name,
                        &args,
                        |slice| Ok(slice.to_vec()),
                    )?)
                } else {
                    bail!("Runtime hasn't been initialized.")
                }
            },
        );
        store.import(
            "plugin",
            HashMap::from([
                ("__modules".to_string(), modules_func),
                ("__call".to_string(), call_func),
            ]),
        )?;

        // fs
        let p = root_path.clone();
        let read_dir_func = store.wrap(move |(path,): (String,)| {
            Ok(p.join(path)?
                .read_dir()?
                .map(|p| p.as_str().to_string())
                .collect::<Vec<_>>())
        });
        store.import(
            "fs",
            HashMap::from([("__read_dir".to_string(), read_dir_func)]),
        )?;

        Ok(store)
    }

    /// Load plugins from specific directory and plugin names.
    ///
    /// The actual load folder will be `rel_to.join(dir)`.
    ///
    /// If `names` is empty, all WASM files will be loaded.
    #[stream(LoadStatus, lifetime = "'a")]
    pub async fn load<'a>(
        dir: impl AsRef<str> + 'a,
        root_path: &'a VfsPath,
        names: &'a [impl AsRef<str>],
    ) -> Result<Arc<Self>> {
        let path = root_path.join(dir)?;
        let paths = if names.is_empty() {
            path.read_dir()?
                .filter_map(|p| {
                    if p.is_file().unwrap_or_default()
                        && p.extension().unwrap_or_default() == "wasm"
                    {
                        let name = p
                            .filename()
                            .strip_suffix(".wasm")
                            .unwrap_or_default()
                            .to_string();
                        Some((name, p))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        } else {
            names
                .iter()
                .filter_map(|name| {
                    let name = name.as_ref();
                    let p = path.join(format!("{}.wasm", name)).unwrap();
                    if p.exists().unwrap_or_default() {
                        Some((name.to_string(), p))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };

        yield LoadStatus::CreateEngine;
        let handle = Arc::new(RwLock::new(Weak::new()));
        let store = Self::new_linker(root_path, handle.clone())?;
        let mut runtime = Self::new();

        let total_len = paths.len();
        for (i, (name, p)) in paths.into_iter().enumerate() {
            yield LoadStatus::LoadPlugin(name.clone(), i, total_len);
            let mut buf = vec![];
            p.open_file()?.read_to_end(&mut buf)?;
            let module = Module::new(store.create(&buf)?);
            runtime.insert_module(name, module)?;
        }
        let runtime = Arc::new(runtime);
        *handle.write().unwrap() = Arc::downgrade(&runtime);
        Ok(runtime)
    }

    fn new() -> Self {
        Self {
            modules: HashMap::default(),
            action_modules: vec![],
            text_modules: HashMap::default(),
            line_modules: HashMap::default(),
            game_modules: vec![],
        }
    }

    fn insert_module(&mut self, name: String, module: Module<M>) -> Result<()> {
        let plugin_type =
            unwrap_or_default_log!(module.plugin_type(), "Cannot determine module type");
        if plugin_type.action {
            self.action_modules.push(name.clone());
        }
        for cmd in plugin_type.text {
            let res = self.text_modules.insert(cmd.clone(), name.clone());
            if let Some(old_module) = res {
                log::warn!(
                    "Text command `{}` is overrided by \"{}\" over \"{}\"",
                    cmd,
                    name,
                    old_module
                );
            }
        }
        for cmd in plugin_type.line {
            let res = self.line_modules.insert(cmd.clone(), name.clone());
            if let Some(old_module) = res {
                log::warn!(
                    "Line command `{}` is overrided by \"{}\" over \"{}\"",
                    cmd,
                    name,
                    old_module
                );
            }
        }
        if plugin_type.game {
            self.game_modules.push(name.clone());
        }
        self.modules.insert(name, module);
        Ok(())
    }

    /// Gets module from name.
    pub fn module(&self, key: &str) -> Option<&Module<M>> {
        self.modules.get(key)
    }

    /// Iterates action modules.
    pub fn action_modules(&self) -> impl Iterator<Item = &Module<M>> {
        self.action_modules
            .iter()
            .map(|key| self.module(key).unwrap())
    }

    /// Gets text module from command.
    pub fn text_module(&self, cmd: &str) -> Option<&Module<M>> {
        self.text_modules.get(cmd).and_then(|key| self.module(key))
    }

    /// Gets line module from command.
    pub fn line_module(&self, cmd: &str) -> Option<&Module<M>> {
        self.line_modules.get(cmd).and_then(|key| self.module(key))
    }

    /// Iterates game modules.
    pub fn game_modules(&self) -> impl Iterator<Item = &Module<M>> {
        self.game_modules
            .iter()
            .map(|key| self.module(key).unwrap())
    }
}

#[doc(hidden)]
pub use backend::BackendModule;

#[doc(hidden)]
mod backend {
    cfg_if::cfg_if! {
        if #[cfg(feature = "wasmi")] {
            pub use ayaka_plugin_wasmi::WasmiModule as BackendModule;
        } else if #[cfg(feature = "wasmtime")] {
            pub use ayaka_plugin_wasmtime::WasmtimeModule as BackendModule;
        } else if #[cfg(feature = "wasmer")] {
            pub use ayaka_plugin_wasmer::WasmerModule as BackendModule;
        } else {
            pub use ayaka_plugin_nop::NopModule as BackendModule;
        }
    }
}
