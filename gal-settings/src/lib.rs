use anyhow::Result;
use gal_locale::Locale;
use gal_primitive::RawValue;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

#[derive(Debug, Default, Deserialize, Serialize)]
struct Settings {
    lang: Locale,
}

pub type VarMap = HashMap<String, RawValue>;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct RawContext {
    pub cur_para: String,
    pub cur_act: usize,
    pub locals: VarMap,
    pub history: Vec<ActionHistoryData>,
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
    let buffer = serde_json::to_vec_pretty(data)?;
    tokio::fs::write(path, &buffer).await?;
    Ok(())
}
