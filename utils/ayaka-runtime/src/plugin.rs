//! The plugin utilities.

#![allow(unsafe_code)]

use crate::*;
use anyhow::Result;
use ayaka_plugin::*;
use std::{collections::HashMap, path::Path};
use stream_future::stream;
use tryiterator::TryIteratorExt;

/// The plugin module with high-level interfaces.
pub struct HostModule<M: RawModule> {
    module: PluginModule<M>,
}

impl<M: RawModule> HostModule<M> {
    fn new(module: M) -> Self {
        Self {
            module: PluginModule::new(module),
        }
    }

    /// Calls a script plugin method by name.
    pub fn dispatch_method(&self, name: &str, args: &[RawValue]) -> Result<RawValue> {
        self.module.call(name, (args,))
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
pub struct HostRuntime<M: RawModule> {
    modules: HashMap<String, HostModule<M>>,
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

impl<M: RawModule> HostRuntime<M> {
    fn new_linker(root_path: impl AsRef<Path>) -> Result<M::Linker> {
        let mut store = M::Linker::new(root_path)?;
        let log_func = store.wrap_with_args(|data: Record| {
            log::logger().log(
                &log::Record::builder()
                    .level(data.level)
                    .target(&data.target)
                    .args(format_args!("{}", data.msg))
                    .module_path(data.module_path.as_deref())
                    .file(data.file.as_deref())
                    .line(data.line)
                    .build(),
            )
        });
        let log_flush_func = store.wrap(|| log::logger().flush());
        store.import(
            "log",
            HashMap::from([
                ("__log".to_string(), log_func),
                ("__log_flush".to_string(), log_flush_func),
            ]),
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
        dir: impl AsRef<Path> + 'a,
        rel_to: impl AsRef<Path> + 'a,
        names: &'a [impl AsRef<str>],
    ) -> Result<Self> {
        let root_path = rel_to.as_ref();
        let path = root_path.join(dir);
        yield LoadStatus::CreateEngine;
        let store = Self::new_linker(root_path)?;
        let mut runtime = Self::new();
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
            let module = HostModule::new(store.create(&buf)?);
            runtime.insert_module(name, module)?;
        }
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

    fn insert_module(&mut self, name: String, module: HostModule<M>) -> Result<()> {
        let plugin_type = module.plugin_type()?;
        if plugin_type.action {
            self.action_modules.push(name.clone());
        }
        for cmd in plugin_type.text {
            let res = self.text_modules.insert(cmd.clone(), name.clone());
            if let Some(old_module) = res {
                log::warn!(
                    "Command `{}` is overrided by \"{}\" over \"{}\"",
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
                    "Command `{}` is overrided by \"{}\" over \"{}\"",
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
    pub fn module(&self, key: &str) -> Option<&HostModule<M>> {
        self.modules.get(key)
    }

    /// Iterates action modules.
    pub fn action_modules(&self) -> impl Iterator<Item = &HostModule<M>> {
        self.action_modules
            .iter()
            .map(|key| self.module(key).unwrap())
    }

    /// Gets text module from command.
    pub fn text_module(&self, cmd: &str) -> Option<&HostModule<M>> {
        self.text_modules.get(cmd).and_then(|key| self.module(key))
    }

    /// Gets line module from command.
    pub fn line_module(&self, cmd: &str) -> Option<&HostModule<M>> {
        self.line_modules.get(cmd).and_then(|key| self.module(key))
    }

    /// Iterates game modules.
    pub fn game_modules(&self) -> impl Iterator<Item = &HostModule<M>> {
        self.game_modules
            .iter()
            .map(|key| self.module(key).unwrap())
    }
}

/// The plugin runtime used in public.
pub type Runtime = HostRuntime<BackendModule>;

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
