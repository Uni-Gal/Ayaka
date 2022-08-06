pub use gal_bindings_types::VarMap;

use crate::*;
use anyhow::{anyhow, Result};
use dirs::{config_dir, data_local_dir};
use gal_locale::LocaleBuf;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio_stream::{wrappers::ReadDirStream, StreamExt};

/// The settings of the game.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Settings {
    /// The display language.
    pub lang: LocaleBuf,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            lang: Locale::current().to_owned(),
        }
    }
}

/// The global record.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct GlobalRecord {
    /// The key is the tag of paragraphs,
    /// the value is the maximum text index.
    pub record: HashMap<String, usize>,
}

/// The serializable context, the record to be saved and loaded.
///
/// To push an [`Action`] into [`RawContext`],
/// you should not use `Vec::push`.
/// Instead, use `RawContext::push_history`.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct RawContext {
    /// Current paragraph tag.
    pub cur_para: String,
    /// Current text index.
    pub cur_act: usize,
    /// Current local variables.
    pub locals: VarMap,
    /// The history actions.
    pub history: Vec<Action>,
}

async fn load_file<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
    let buffer = tokio::fs::read(path).await?;
    Ok(serde_json::from_slice(&buffer)?)
}

async fn save_file<T: Serialize>(data: &T, path: impl AsRef<Path>, pretty: bool) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let buffer = if pretty {
        serde_json::to_vec_pretty(data)
    } else {
        serde_json::to_vec(data)
    }?;
    tokio::fs::write(path, &buffer).await?;
    Ok(())
}

fn settings_path(ident: &str) -> Result<PathBuf> {
    let path = config_dir().ok_or_else(|| anyhow!("Cannot find config path"))?;
    Ok(path.join(ident).join("settings.json"))
}

/// Load settings from JSON file.
pub async fn load_settings(ident: &str) -> Result<Settings> {
    load_file(settings_path(ident)?).await
}

/// Save settings into pretty JSON file.
pub async fn save_settings(ident: &str, data: &Settings) -> Result<()> {
    save_file(data, settings_path(ident)?, true).await
}

fn records_path(ident: &str, game: &str) -> Result<PathBuf> {
    let path = data_local_dir().ok_or_else(|| anyhow!("Cannot find config path"))?;
    Ok(path.join(ident).join("save").join(game))
}

fn global_record_path(ident: &str, game: &str) -> Result<PathBuf> {
    Ok(records_path(ident, game)?.join("global.json"))
}

pub async fn load_global_record(ident: &str, game: &str) -> Result<GlobalRecord> {
    load_file(global_record_path(ident, game)?).await
}

pub async fn save_global_record(ident: &str, game: &str, data: &GlobalRecord) -> Result<()> {
    save_file(data, global_record_path(ident, game)?, false).await
}

/// Load all [`RawContext`] from the records folder.
pub async fn load_records(ident: &str, game: &str) -> Result<Vec<RawContext>> {
    let ctx_path = records_path(ident, game)?;
    let mut entries = ReadDirStream::new(tokio::fs::read_dir(ctx_path).await?);
    let mut contexts = vec![];
    while let Some(entry) = entries.try_next().await? {
        let p = entry.path();
        if p.extension()
            .map(|s| s.to_string_lossy())
            .unwrap_or_default()
            == "json"
        {
            contexts.push(load_file(&p).await?);
        }
    }
    Ok(contexts)
}

/// Save all [`RawContext`] into the records folder.
pub async fn save_records(ident: &str, game: &str, contexts: &[RawContext]) -> Result<()> {
    let ctx_path = records_path(ident, game)?;
    for (i, ctx) in contexts.iter().enumerate() {
        save_file(
            ctx,
            ctx_path.join(i.to_string()).with_extension("json"),
            false,
        )
        .await?;
    }
    Ok(())
}
