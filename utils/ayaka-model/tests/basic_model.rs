#![feature(return_position_impl_trait_in_trait)]
#![allow(incomplete_features)]

use ayaka_model::{
    anyhow::{Error, Result},
    *,
};
use ayaka_plugin_wasmi::{WasmiLinker, WasmiModule};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    path::{Path, PathBuf},
    pin::Pin,
};
use tempfile::{tempdir, TempDir};

struct NopSettingsManager {
    dir: TempDir,
}

impl NopSettingsManager {
    pub fn new() -> Result<Self> {
        Ok(Self { dir: tempdir()? })
    }

    fn records_path_root(&self, game: &str) -> PathBuf {
        self.dir.path().join("save").join(game)
    }
}

impl SettingsManager for NopSettingsManager {
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
        Ok(self.dir.path().join("settings.json"))
    }

    fn global_record_path(&self, game: &str) -> Result<PathBuf> {
        Ok(self.records_path_root(game).join("global.json"))
    }

    fn records_path(&self, game: &str) -> Result<impl Iterator<Item = Result<PathBuf>>> {
        let ctx_path = self.records_path_root(game);
        Ok(std::fs::read_dir(ctx_path)?.filter_map(|entry| {
            entry
                .map_err(Error::from)
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

const CONFIG_PATH: &str = "tests/basic/config.yaml";

#[tokio::test(flavor = "current_thread")]
async fn progress() {
    let mut model =
        GameViewModel::<NopSettingsManager, WasmiModule>::new(NopSettingsManager::new().unwrap());
    let linker = WasmiLinker::new(()).unwrap();
    let context = ContextBuilder::<WasmiModule>::new(FrontendType::Text, linker)
        .with_paths(&[CONFIG_PATH])
        .unwrap()
        .open()
        .await
        .unwrap();
    let mut context = model.open_game(context);
    let progresses = unsafe { Pin::new_unchecked(&mut context) }
        .collect::<Vec<_>>()
        .await;
    context.await.unwrap();
    assert_eq!(
        &progresses,
        &[
            OpenGameStatus::LoadSettings,
            OpenGameStatus::LoadGlobalRecords,
            OpenGameStatus::LoadRecords,
            OpenGameStatus::Loaded,
        ]
    );
}

fn text_chars(s: impl Into<String>) -> Action {
    let mut text = ActionText::default();
    text.push_back_chars(s.into());
    Action::Text(text)
}

#[tokio::test(flavor = "current_thread")]
async fn paras() {
    let manager = {
        let settings = Settings {
            lang: locale!("en"),
            sub_lang: Some(locale!("zh")),
            ..Default::default()
        };
        let manager = NopSettingsManager::new().unwrap();
        manager.save_settings(&settings).unwrap();
        manager
    };
    let mut model = GameViewModel::<NopSettingsManager, WasmiModule>::new(manager);
    let linker = WasmiLinker::new(()).unwrap();
    let context = ContextBuilder::<WasmiModule>::new(FrontendType::Text, linker)
        .with_paths(&[CONFIG_PATH])
        .unwrap()
        .open()
        .await
        .unwrap();
    model.open_game(context).await.unwrap();
    model.init_new();
    let actions = std::iter::from_fn(|| {
        if model.next_run() {
            model.current_actions()
        } else {
            None
        }
    })
    .collect::<Vec<_>>();
    assert_eq!(
        &actions,
        &[
            (text_chars("0"), Some(text_chars("0"))),
            (text_chars("1"), Some(text_chars("114514"))),
            (text_chars("2"), Some(text_chars("2"))),
            (text_chars("3"), Some(text_chars("abcdef"))),
        ]
    )
}
