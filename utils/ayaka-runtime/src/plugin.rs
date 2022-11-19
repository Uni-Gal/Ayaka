//! The plugin utilities.

#![allow(unsafe_code)]
#![allow(clippy::mut_from_ref)]

use crate::*;
use anyhow::Result;
use ayaka_plugin::*;
use std::{ops::Deref, path::Path};
use stream_future::stream;
use tryiterator::TryIteratorExt;

use ayaka_plugin_wasmer::*;

/// The plugin runtime.
pub struct HostRuntime<M: RawModule> {
    runtime: PluginRuntime<M>,
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
        let store = M::Linker::new(root_path)?;
        let mut runtime = PluginRuntime::new();
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
            let module = PluginModule::new(store.from_binary(&buf)?);
            runtime.insert_module(name, module)?;
        }
        Ok(Self { runtime })
    }
}

impl<M: RawModule> Deref for HostRuntime<M> {
    type Target = PluginRuntime<M>;

    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}

/// The plugin runtime used in public.
pub type Runtime = HostRuntime<WasmerModule>;
