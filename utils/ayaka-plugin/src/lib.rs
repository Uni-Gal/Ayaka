//! Base crate for plugin runtimes.

#![feature(fn_traits)]
#![feature(tuple_trait)]
#![feature(type_alias_impl_trait)]
#![feature(unboxed_closures)]
#![warn(missing_docs)]

pub use anyhow::Result;

use ayaka_bindings_types::*;
use ayaka_script::{log, RawValue};
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashMap, marker::Tuple, path::Path};

/// Represents a raw plugin module.
pub trait RawModule: Sized {
    /// The linker type that can create raw module.
    type Linker: StoreLinker<Self>;

    /// The import function type.
    type Func;

    /// Calls a method by name.
    ///
    /// The args and returns are passed by MessagePack with [`rmp_serde`].
    fn call<T>(&self, name: &str, args: &[u8], f: impl FnOnce(&[u8]) -> Result<T>) -> Result<T>;
}

/// High-level wrapper for plugin module.
pub struct PluginModule<M: RawModule> {
    module: M,
}

impl<M: RawModule> PluginModule<M> {
    /// Creates a wrapper on raw module.
    pub fn new(module: M) -> Self {
        Self { module }
    }

    fn call<P: Serialize, R: DeserializeOwned>(&self, name: &str, args: P) -> Result<R> {
        let data = rmp_serde::to_vec(&args)?;
        self.module.call(name, &data, |res| {
            let res = rmp_serde::from_slice(res)?;
            Ok(res)
        })
    }

    /// Calls a script plugin method by name.
    pub fn dispatch_method(&self, name: &str, args: &[RawValue]) -> Result<RawValue> {
        self.call(name, (args,))
    }

    /// Gets the [`PluginType`].
    pub fn plugin_type(&self) -> Result<PluginType> {
        self.call("plugin_type", ())
    }

    /// Processes [`Action`] in action plugin.
    pub fn process_action(&self, ctx: ActionProcessContextRef) -> Result<ActionProcessResult> {
        self.call("process_action", (ctx,))
    }

    /// Calls a custom command in the text plugin.
    pub fn dispatch_text(
        &self,
        name: &str,
        args: &[String],
        ctx: TextProcessContextRef,
    ) -> Result<TextProcessResult> {
        self.call(name, (args, ctx))
    }

    /// Calls a custom command in the line plugin.
    pub fn dispatch_line(
        &self,
        name: &str,
        ctx: LineProcessContextRef,
    ) -> Result<LineProcessResult> {
        self.call(name, (ctx,))
    }

    /// Processes [`Game`] when opening the config file.
    pub fn process_game(&self, ctx: GameProcessContextRef) -> Result<GameProcessResult> {
        self.call("process_game", (ctx,))
    }
}

/// Represents the store & linker of plugin modules.
pub trait StoreLinker<M: RawModule>: Sized {
    /// Creates a new instance of [`StoreLinker`].
    ///
    /// The `root_path` is used to preopen the root dir,
    /// and mapped to `/`.
    fn new(root_path: impl AsRef<Path>) -> Result<Self>;

    /// Create a raw module from binary.
    fn create(&self, binary: &[u8]) -> Result<M>;

    /// Import functions by namespace and names.
    fn import(&mut self, ns: impl Into<String>, funcs: HashMap<String, M::Func>) -> Result<()>;

    /// Wrap a simple function.
    fn wrap(&self, f: impl Fn() + Send + Sync + 'static) -> M::Func;

    /// Wrap a function with args in bytes.
    fn wrap_with_args_raw(
        &self,
        f: impl (Fn(*const [u8]) -> Result<()>) + Send + Sync + 'static,
    ) -> M::Func;

    /// Wrap a function with args.
    fn wrap_with_args<Params: DeserializeOwned + Tuple>(
        &self,
        f: impl Fn<Params, Output = ()> + Send + Sync + 'static,
    ) -> M::Func {
        self.wrap_with_args_raw(move |data| {
            let data = rmp_serde::from_slice(unsafe { &*data })?;
            f.call(data);
            Ok(())
        })
    }
}

/// The plugin runtime.
pub struct PluginRuntime<M: RawModule> {
    modules: HashMap<String, PluginModule<M>>,
    action_modules: Vec<String>,
    text_modules: HashMap<String, String>,
    line_modules: HashMap<String, String>,
    game_modules: Vec<String>,
}

impl<M: RawModule> Default for PluginRuntime<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: RawModule> PluginRuntime<M> {
    /// Create the runtime from a store.
    pub fn new() -> Self {
        Self {
            modules: HashMap::default(),
            action_modules: vec![],
            text_modules: HashMap::default(),
            line_modules: HashMap::default(),
            game_modules: vec![],
        }
    }

    /// Insert a plugin with binary.
    pub fn insert_module(&mut self, name: String, module: PluginModule<M>) -> Result<()> {
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
}

/// Represents a plugin resolver.
pub trait PluginResolver {
    /// The type of raw plugin module.
    type Module: RawModule;

    /// Gets module from name.
    fn module(&self, key: &str) -> Option<&PluginModule<Self::Module>>;

    #[doc(hidden)]
    type ActionMIter<'a>: Iterator<Item = &'a PluginModule<Self::Module>>
    where
        Self: 'a;

    /// Iterates action modules.
    fn action_modules<'a>(&'a self) -> Self::ActionMIter<'a>;

    /// Gets text module from command.
    fn text_module(&self, cmd: &str) -> Option<&PluginModule<Self::Module>>;

    /// Gets line module from command.
    fn line_module(&self, cmd: &str) -> Option<&PluginModule<Self::Module>>;

    #[doc(hidden)]
    type GameMIter<'a>: Iterator<Item = &'a PluginModule<Self::Module>>
    where
        Self: 'a;

    /// Iterates game modules.
    fn game_modules<'a>(&'a self) -> Self::GameMIter<'a>;
}

impl<M: RawModule> PluginResolver for PluginRuntime<M> {
    type Module = M;

    fn module(&self, key: &str) -> Option<&PluginModule<M>> {
        self.modules.get(key)
    }

    type ActionMIter<'a> = impl Iterator<Item = &'a PluginModule<M>>
    where
        M: 'a;

    fn action_modules<'a>(&'a self) -> Self::ActionMIter<'a> {
        self.action_modules
            .iter()
            .map(|key| self.module(key).unwrap())
    }

    fn text_module(&self, cmd: &str) -> Option<&PluginModule<M>> {
        self.text_modules.get(cmd).and_then(|key| self.module(key))
    }

    fn line_module(&self, cmd: &str) -> Option<&PluginModule<M>> {
        self.line_modules.get(cmd).and_then(|key| self.module(key))
    }

    type GameMIter<'a> = impl Iterator<Item = &'a PluginModule<M>>
    where
        M: 'a;

    fn game_modules<'a>(&'a self) -> Self::GameMIter<'a> {
        self.game_modules
            .iter()
            .map(|key| self.module(key).unwrap())
    }
}
