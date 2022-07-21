use gal_fallback::FallbackSpec;
use gal_script::Program;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    path::{Path, PathBuf},
};

#[doc(hidden)]
#[derive(Serialize, Deserialize)]
pub struct Record {
    pub level: log::Level,
    pub target: String,
    pub msg: String,
    pub module_path: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
}

impl From<&log::Record<'_>> for Record {
    fn from(r: &log::Record) -> Self {
        Self {
            level: r.level(),
            target: r.target().to_string(),
            msg: r.args().to_string(),
            module_path: r.module_path().map(|s| s.to_string()),
            file: r.file().map(|s| s.to_string()),
            line: r.line(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PluginType {
    Script,
    Action,
    Text,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FrontendType {
    Text,
    Html,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ActionLine {
    Chars(String),
    Block(String),
}

impl ActionLine {
    pub fn chars(s: impl Into<String>) -> Self {
        Self::Chars(s.into())
    }

    pub fn block(s: impl Into<String>) -> Self {
        Self::Block(s.into())
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Chars(s) | Self::Block(s) => &s,
        }
    }

    pub fn into_string(self) -> String {
        match self {
            Self::Chars(s) | Self::Block(s) => s,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, FallbackSpec)]
pub struct Action {
    pub line: VecDeque<ActionLine>,
    pub character: Option<String>,
    pub para_title: Option<String>,
    pub switches: Vec<Switch>,
    pub props: HashMap<String, String>,
    pub switch_actions: Vec<Program>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, FallbackSpec)]
pub struct Switch {
    pub text: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextProcessContext {
    pub root_path: PathBuf,
    pub game_props: HashMap<String, String>,
    pub frontend: FrontendType,
}

impl TextProcessContext {
    pub fn as_ref<'a>(&'a self) -> TextProcessContextRef<'a> {
        TextProcessContextRef {
            root_path: &self.root_path,
            game_props: &self.game_props,
            frontend: self.frontend,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TextProcessContextRef<'a> {
    pub root_path: &'a Path,
    pub game_props: &'a HashMap<String, String>,
    pub frontend: FrontendType,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TextProcessResult {
    pub line: VecDeque<ActionLine>,
    pub props: HashMap<String, String>,
}
