use anyhow::{anyhow, Result};
use dirs::{config_dir, data_dir};
use gal_locale::Locale;
use gal_primitive::RawValue;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio_stream::{wrappers::ReadDirStream, StreamExt};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Settings {
    lang: Locale,
}

pub type VarMap = HashMap<String, RawValue>;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct RawContext {
    pub cur_para: String,
    pub cur_act: usize,
    pub locals: VarMap,
    pub history: Vec<ActionHistoryData>,
    pub bg: Option<String>,
    pub bgm: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ActionHistoryData {
    pub line: String,
    pub character: Option<String>,
}

pub async fn load_file<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
    let buffer = tokio::fs::read(path).await?;
    Ok(serde_json::from_slice(&buffer)?)
}

pub async fn save_file<T: Serialize>(data: &T, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let buffer = serde_json::to_vec_pretty(data)?;
    tokio::fs::write(path, &buffer).await?;
    Ok(())
}

pub fn settings_path() -> Result<PathBuf> {
    let path = config_dir().ok_or_else(|| anyhow!("Cannot find config path"))?;
    Ok(path.join("gal").join("settings.json"))
}

pub async fn load_settings() -> Result<Settings> {
    load_file(settings_path()?).await
}

pub async fn save_settings(data: &Settings) -> Result<()> {
    save_file(data, settings_path()?).await
}

pub fn context_path() -> Result<PathBuf> {
    let path = data_dir().ok_or_else(|| anyhow!("Cannot find config path"))?;
    Ok(path.join("gal").join("save"))
}

pub async fn load_records() -> Result<Vec<RawContext>> {
    let ctx_path = context_path()?;
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

pub async fn save_records(contexts: &[RawContext]) -> Result<()> {
    let ctx_path = context_path()?;
    for (i, ctx) in contexts.iter().enumerate() {
        save_file(ctx, ctx_path.join(i.to_string()).with_extension("json")).await?;
    }
    Ok(())
}
