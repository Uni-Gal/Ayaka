use ayaka_runtime::{
    anyhow::{self, Result},
    settings::*,
};
use serde::{de::DeserializeOwned, Serialize};
use std::path::{Path, PathBuf};
use tauri::PathResolver;
use tryiterator::TryIteratorExt;

#[derive(Default)]
pub struct FileSettingsManager {
    local_data_dir: PathBuf,
    config_dir: PathBuf,
}

impl FileSettingsManager {
    pub fn new(resolver: &PathResolver) -> Self {
        Self {
            local_data_dir: resolver.app_local_data_dir().unwrap(),
            config_dir: resolver.app_config_dir().unwrap(),
        }
    }

    fn records_path_root(&self, game: &str) -> PathBuf {
        self.local_data_dir.join("save").join(game)
    }
}

impl SettingsManager for FileSettingsManager {
    fn load_file<T: DeserializeOwned>(&self, path: impl AsRef<Path>) -> Result<T> {
        let buffer = std::fs::read(path)?;
        Ok(serde_json::from_slice(&buffer)?)
    }

    fn save_file<T: Serialize>(
        &self,
        path: impl AsRef<Path>,
        data: &T,
        pretty: bool,
    ) -> Result<()> {
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

    fn settings_path(&self) -> Result<PathBuf> {
        Ok(self.config_dir.join("settings.json"))
    }

    fn global_record_path(&self, game: &str) -> Result<PathBuf> {
        Ok(self.records_path_root(game).join("global.json"))
    }

    type RecordsPathIter = impl Iterator<Item = Result<PathBuf>>;

    fn records_path(&self, game: &str) -> Result<Self::RecordsPathIter> {
        let ctx_path = self.records_path_root(game);
        Ok(std::fs::read_dir(ctx_path)?
            .map_err(anyhow::Error::from)
            .try_filter_map(|entry| {
                let p = entry.path();
                if p.is_file() && p.file_name().unwrap_or_default() != "global.json" {
                    Ok(Some(p))
                } else {
                    Ok(None)
                }
            }))
    }

    fn record_path(&self, game: &str, i: usize) -> Result<PathBuf> {
        Ok(self
            .records_path_root(game)
            .join(i.to_string())
            .with_extension("json"))
    }
}
