use crate::*;
use anyhow::{anyhow, Result};
use ayaka_bindings_types::RawContext;
use dirs::{config_dir, data_local_dir};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tryiterator::TryIteratorExt;

/// The settings of the game.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Settings {
    /// The display language.
    pub lang: Locale,
    /// The secondary display language.
    pub sub_lang: Option<Locale>,
}

/// The global record.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct GlobalRecord {
    /// The key is the tag of paragraphs,
    /// the value is the maximum text index.
    pub record: HashMap<String, usize>,
}

impl GlobalRecord {
    /// Determine if an [`ActionParams`] has been visited,
    /// by the paragraph tag and action index.
    pub fn visited(&self, ctx: &RawContext) -> bool {
        if let Some(max_act) = self.record.get(&ctx.cur_para) {
            log::debug!("Test act: {}, max act: {}", ctx.cur_act, max_act);
            *max_act >= ctx.cur_act
        } else {
            false
        }
    }

    /// Update the global record with the latest [`ActionParams`].
    pub fn update(&mut self, ctx: &RawContext) {
        self.record
            .entry(ctx.cur_para.clone())
            .and_modify(|act| *act = (*act).max(ctx.cur_act))
            .or_insert(ctx.cur_act);
    }
}

/// The specific record.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ActionRecord {
    /// The history actions.
    pub history: Vec<RawContext>,
}

impl ActionRecord {
    /// Get the [`RawContext`] object from the last [`Action`] in the history.
    pub fn last_ctx(&self) -> Option<&RawContext> {
        self.history.last()
    }

    /// Get the [`RawContext`] object from the last [`Action`] in the history,
    /// and if the history is empty, create a new [`RawContext`] from the game.
    pub fn last_ctx_with_game(&self, game: &Game) -> RawContext {
        self.last_ctx().cloned().unwrap_or_else(|| RawContext {
            cur_base_para: game.config.start.clone(),
            cur_para: game.config.start.clone(),
            ..Default::default()
        })
    }
}

fn load_file<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
    let buffer = std::fs::read(path)?;
    Ok(serde_json::from_slice(&buffer)?)
}

fn save_file<T: Serialize>(data: &T, path: impl AsRef<Path>, pretty: bool) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let buffer = if pretty {
        serde_json::to_vec_pretty(data)
    } else {
        serde_json::to_vec(data)
    }?;
    std::fs::write(path, buffer)?;
    Ok(())
}

fn settings_path(ident: &str) -> Result<PathBuf> {
    let path = config_dir().ok_or_else(|| anyhow!("Cannot find config path"))?;
    Ok(path.join(ident).join("settings.json"))
}

/// Load settings from JSON file.
pub fn load_settings(ident: &str) -> Result<Settings> {
    load_file(settings_path(ident)?)
}

/// Save settings into pretty JSON file.
pub fn save_settings(ident: &str, data: &Settings) -> Result<()> {
    save_file(data, settings_path(ident)?, true)
}

fn records_path(ident: &str, game: &str) -> Result<PathBuf> {
    let path = data_local_dir().ok_or_else(|| anyhow!("Cannot find config path"))?;
    Ok(path.join(ident).join("save").join(game))
}

fn global_record_path(ident: &str, game: &str) -> Result<PathBuf> {
    Ok(records_path(ident, game)?.join("global.json"))
}

/// Load [`GlobalRecord`] from the records folder.
pub fn load_global_record(ident: &str, game: &str) -> Result<GlobalRecord> {
    load_file(global_record_path(ident, game)?)
}

/// Save [`GlobalRecord`] into the records folder.
pub fn save_global_record(ident: &str, game: &str, data: &GlobalRecord) -> Result<()> {
    save_file(data, global_record_path(ident, game)?, false)
}

/// Load all [`ActionRecord`] from the records folder.
pub fn load_records(ident: &str, game: &str) -> Result<Vec<ActionRecord>> {
    let ctx_path = records_path(ident, game)?;
    let contexts = std::fs::read_dir(ctx_path)?
        .map_err(anyhow::Error::from)
        .try_filter_map(|entry| {
            let p = entry.path();
            if p.is_file() && p.file_name().unwrap_or_default() != "global.json" {
                Ok(Some(load_file(&p)?))
            } else {
                Ok(None)
            }
        })
        .try_collect()?;
    Ok(contexts)
}

/// Save all [`ActionRecord`] into the records folder.
pub fn save_records(ident: &str, game: &str, contexts: &[ActionRecord]) -> Result<()> {
    let ctx_path = records_path(ident, game)?;
    for (i, ctx) in contexts.iter().enumerate() {
        save_file(
            ctx,
            ctx_path.join(i.to_string()).with_extension("json"),
            false,
        )?;
    }
    Ok(())
}
