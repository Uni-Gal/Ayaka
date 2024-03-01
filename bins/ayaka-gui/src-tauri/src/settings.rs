use ayaka_model::{anyhow::Result, *};
use serde::{de::DeserializeOwned, Serialize};
use std::path::{Path, PathBuf};
use tauri::{App, Manager};

#[derive(Default)]
pub struct FileSettingsManager {
    local_data_dir: PathBuf,
    config_dir: PathBuf,
}

impl FileSettingsManager {
    pub fn new(app: &App) -> Self {
        let resolver = app.path();
        Self {
            local_data_dir: resolver
                .app_local_data_dir()
                .expect("cannot get app local data dir"),
            config_dir: resolver
                .app_config_dir()
                .expect("cannot get app config dir"),
        }
    }

    fn records_path_root(&self, game: &str) -> PathBuf {
        self.local_data_dir.join("save").join(game)
    }
}

impl SettingsManager for FileSettingsManager {
    fn load_file<T: DeserializeOwned>(&self, path: impl AsRef<Path>) -> Result<T> {
        let file = std::fs::File::open(path)?;
        Ok(serde_json::from_reader(file)?)
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
        let output = std::fs::File::create(path)?;
        if pretty {
            serde_json::to_writer_pretty(output, data)
        } else {
            serde_json::to_writer(output, data)
        }?;
        Ok(())
    }

    fn settings_path(&self) -> Result<PathBuf> {
        Ok(self.config_dir.join("settings.json"))
    }

    fn global_record_path(&self, game: &str) -> Result<PathBuf> {
        Ok(self.records_path_root(game).join("global.json"))
    }

    fn records_path(&self, game: &str) -> Result<impl Iterator<Item = Result<PathBuf>>> {
        let ctx_path = self.records_path_root(game);
        Ok(std::fs::read_dir(ctx_path)?.filter_map(|entry| {
            entry
                .map_err(anyhow::Error::from)
                .map(|entry| {
                    let p = entry.path();
                    if p.is_file() && p.file_name().unwrap_or_default() != "global.json" {
                        Some(p)
                    } else {
                        None
                    }
                })
                .transpose()
        }))
    }

    fn record_path(&self, game: &str, i: usize) -> Result<PathBuf> {
        Ok(self
            .records_path_root(game)
            .join(i.to_string())
            .with_extension("json"))
    }
}
