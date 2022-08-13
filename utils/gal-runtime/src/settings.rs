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
    /// Creates [`Settings`] object with current locale.
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

/// The specific record.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ActionRecord {
    /// The history actions.
    pub history: Vec<Action>,
}

impl ActionRecord {
    /// Get the [`RawContext`] object from the last [`Action`] in the history.
    pub fn last_ctx(&self) -> Option<&RawContext> {
        self.history.last().map(|act| &act.ctx)
    }

    /// Get the [`RawContext`] object from the last [`Action`] in the history,
    /// and if the history is empty, create a new [`RawContext`] from the game.
    pub fn last_ctx_with_game(&self, game: &Game) -> RawContext {
        self.last_ctx().cloned().unwrap_or_else(|| RawContext {
            cur_para: game
                .paras
                .get(&game.base_lang)
                .and_then(|paras| paras.first().map(|p| p.tag.clone()))
                .unwrap_or_else(|| {
                    log::warn!("There is no paragraph in the game.");
                    Default::default()
                }),
            ..Default::default()
        })
    }
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

/// Load [`GlobalRecord`] from the records folder.
pub async fn load_global_record(ident: &str, game: &str) -> Result<GlobalRecord> {
    load_file(global_record_path(ident, game)?).await
}

/// Save [`GlobalRecord`] into the records folder.
pub async fn save_global_record(ident: &str, game: &str, data: &GlobalRecord) -> Result<()> {
    save_file(data, global_record_path(ident, game)?, false).await
}

/// Load all [`ActionRecord`] from the records folder.
pub async fn load_records(ident: &str, game: &str) -> Result<Vec<ActionRecord>> {
    let ctx_path = records_path(ident, game)?;
    let mut entries = ReadDirStream::new(tokio::fs::read_dir(ctx_path).await?);
    let mut contexts = vec![];
    while let Some(entry) = entries.try_next().await? {
        let p = entry.path();
        if p.extension()
            .map(|s| s.to_string_lossy())
            .unwrap_or_default()
            == "json"
            && p.file_stem()
                .map(|s| s.to_string_lossy())
                .unwrap_or_default()
                != "global"
        {
            contexts.push(load_file(&p).await?);
        }
    }
    Ok(contexts)
}

/// Save all [`ActionRecord`] into the records folder.
pub async fn save_records(ident: &str, game: &str, contexts: &[ActionRecord]) -> Result<()> {
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
