use std::collections::HashMap;

use crate::*;
use anyhow::Result;
use ayaka_script::RawValue;
use serde::{de::DeserializeOwned, Serialize};

pub trait PluginModule {
    /// Calls a method by name.
    ///
    /// The args and returns are passed by MessagePack with [`rmp_serde`].
    fn call<P: Serialize, R: DeserializeOwned>(&self, name: &str, args: P) -> Result<R>;

    /// Calls a script plugin method by name.
    fn dispatch_method(&self, name: &str, args: &[RawValue]) -> Result<RawValue> {
        self.call(name, (args,))
    }

    /// Gets the [`PluginType`].
    fn plugin_type(&self) -> Result<PluginType> {
        self.call("plugin_type", ())
    }

    /// Processes [`Action`] in action plugin.
    fn process_action(&self, ctx: ActionProcessContextRef) -> Result<ActionProcessResult> {
        self.call("process_action", (ctx,))
    }

    /// Calls a custom command in the text plugin.
    fn dispatch_text(
        &self,
        name: &str,
        args: &[String],
        ctx: TextProcessContextRef,
    ) -> Result<TextProcessResult> {
        self.call(name, (args, ctx))
    }

    /// Calls a custom command in the line plugin.
    fn dispatch_line(&self, name: &str, ctx: LineProcessContextRef) -> Result<LineProcessResult> {
        self.call(name, (ctx,))
    }

    /// Processes [`Game`] when opening the config file.
    fn process_game(&self, ctx: GameProcessContextRef) -> Result<GameProcessResult> {
        self.call("process_game", (ctx,))
    }
}

pub trait PluginModuleStore {
    type Module: PluginModule;

    fn from_binary(&self, binary: &[u8]) -> Result<Self::Module>;
}

pub struct PluginRuntime<M: PluginModule, S: PluginModuleStore<Module = M>> {
    store: S,
    modules: HashMap<String, M>,
    action_modules: Vec<String>,
    text_modules: HashMap<String, String>,
    line_modules: HashMap<String, String>,
    game_modules: Vec<String>,
}

impl<M: PluginModule, S: PluginModuleStore<Module = M>> PluginRuntime<M, S> {
    pub fn new(store: S) -> Self {
        Self {
            store,
            modules: HashMap::default(),
            action_modules: vec![],
            text_modules: HashMap::default(),
            line_modules: HashMap::default(),
            game_modules: vec![],
        }
    }

    pub fn insert_binary(&mut self, name: String, binary: &[u8]) -> Result<()> {
        let module = self.store.from_binary(binary)?;
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

pub trait PluginContext {
    type Module: PluginModule;

    fn get_module(&self, name: &str) -> Option<&Self::Module>;

    fn find_text_module(&self, cmd: &str) -> Option<&Self::Module>;

    fn find_line_module(&self, cmd: &str) -> Option<&Self::Module>;

    type ActionMIter<'a>: Iterator<Item = &'a Self::Module>
    where
        Self: 'a;

    fn action_modules<'a>(&'a self) -> Self::ActionMIter<'a>;

    type GameMIter<'a>: Iterator<Item = &'a Self::Module>
    where
        Self: 'a;

    fn game_modules<'a>(&'a self) -> Self::GameMIter<'a>;
}

impl<M: PluginModule, S: PluginModuleStore<Module = M>> PluginContext for PluginRuntime<M, S> {
    type Module = M;

    fn get_module(&self, name: &str) -> Option<&M> {
        self.modules.get(name)
    }

    fn find_text_module(&self, cmd: &str) -> Option<&M> {
        self.text_modules.get(cmd).map(|name| &self.modules[name])
    }

    fn find_line_module(&self, cmd: &str) -> Option<&M> {
        self.line_modules.get(cmd).map(|name| &self.modules[name])
    }

    type ActionMIter<'a> = impl Iterator<Item = &'a M>
    where
        Self:'a;

    fn action_modules<'a>(&'a self) -> Self::ActionMIter<'a> {
        self.action_modules.iter().map(|name| &self.modules[name])
    }

    type GameMIter<'a> = impl Iterator<Item = &'a M>
    where
        Self:'a;

    fn game_modules<'a>(&'a self) -> Self::GameMIter<'a> {
        self.game_modules.iter().map(|name| &self.modules[name])
    }
}
