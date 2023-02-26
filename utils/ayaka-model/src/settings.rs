use crate::*;
use anyhow::Result;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

/// The settings of the game.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    /// The display language.
    pub lang: Locale,
    /// The secondary display language.
    pub sub_lang: Option<Locale>,
    /// Volume of background music.
    pub bgm_volume: u8,
    /// Volume of voices.
    pub voice_volume: u8,
    /// Volume of videos.
    pub video_volume: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            lang: Locale::default(),
            sub_lang: None,
            bgm_volume: 100,
            voice_volume: 100,
            video_volume: 100,
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

impl GlobalRecord {
    /// Determine if an [`RawContext`] has been visited,
    /// by the paragraph tag and action index.
    pub fn visited(&self, ctx: &RawContext) -> bool {
        if let Some(max_act) = self.record.get(&ctx.cur_para) {
            log::debug!("Test act: {}, max act: {}", ctx.cur_act, max_act);
            *max_act >= ctx.cur_act
        } else {
            false
        }
    }

    /// Update the global record with the latest [`RawContext`].
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
        self.last_ctx()
            .cloned()
            .unwrap_or_else(|| game.start_context())
    }
}

/// A settings manager trait.
///
/// This type should handle the file loading and saving,
/// and manage the paths of the files.
pub trait SettingsManager {
    /// Load a file from specified path.
    fn load_file<T: DeserializeOwned>(&self, path: impl AsRef<Path>) -> Result<T>;

    /// Save a file to specified path.
    fn save_file<T: Serialize>(&self, path: impl AsRef<Path>, data: &T, pretty: bool)
        -> Result<()>;

    /// Get the settings path.
    fn settings_path(&self) -> Result<PathBuf>;

    /// Load [`Settings`].
    fn load_settings(&self) -> Result<Settings> {
        self.load_file(self.settings_path()?)
    }

    /// Save [`Settings`].
    fn save_settings(&self, data: &Settings) -> Result<()> {
        self.save_file(self.settings_path()?, data, true)
    }

    /// Get the global record path.
    fn global_record_path(&self, game: &str) -> Result<PathBuf>;

    /// Load [`GlobalRecord`].
    fn load_global_record(&self, game: &str) -> Result<GlobalRecord> {
        self.load_file(self.global_record_path(game)?)
    }

    /// Save [`GlobalRecord`].
    fn save_global_record(&self, game: &str, data: &GlobalRecord) -> Result<()> {
        self.save_file(self.global_record_path(game)?, data, false)
    }

    /// Get an iterator of record paths.
    fn records_path(&self, game: &str) -> Result<impl Iterator<Item = Result<PathBuf>>>;

    /// Get the record path from index.
    fn record_path(&self, game: &str, i: usize) -> Result<PathBuf>;

    /// Load all [`ActionRecord`].
    fn load_records(&self, game: &str) -> Result<Vec<ActionRecord>> {
        self.records_path(game)?
            .map(|path| path.and_then(|path| self.load_file(path)))
            .collect()
    }

    /// Save all [`ActionRecord`].
    fn save_records(&self, game: &str, contexts: &[ActionRecord]) -> Result<()> {
        for (i, ctx) in contexts.iter().enumerate() {
            self.save_file(self.record_path(game, i)?, ctx, false)?;
        }
        Ok(())
    }
}
