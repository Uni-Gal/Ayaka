use gal_fallback::{FallbackSpec, IsEmpty2};
use gal_script::Program;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
    ops::{Deref, DerefMut},
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

bitflags::bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct PluginType: u32 {
        const SCRIPT = 0;
        const ACTION = 0b1;
        const TEXT = 0b10;
    }
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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActionLines(VecDeque<ActionLine>);

impl Deref for ActionLines {
    type Target = VecDeque<ActionLine>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ActionLines {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for ActionLines {
    type Item = ActionLine;

    type IntoIter = <VecDeque<ActionLine> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl ActionLines {
    pub fn push_back_chars<'a>(&mut self, s: impl Into<Cow<'a, str>>) {
        let s = s.into();
        if let Some(act) = self.back_mut() {
            if let ActionLine::Chars(text) = act {
                text.push_str(&s);
                return;
            }
        }
        self.push_back(ActionLine::Chars(s.into_owned()))
    }

    pub fn push_back_block<'a>(&mut self, s: impl Into<Cow<'a, str>>) {
        let s = s.into();
        if let Some(act) = self.back_mut() {
            if let ActionLine::Block(text) = act {
                text.push_str(&s);
                return;
            }
        }
        self.push_back(ActionLine::Block(s.into_owned()))
    }
}

impl IsEmpty2 for ActionLines {
    fn is_empty2(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, FallbackSpec)]
pub struct Action {
    pub line: ActionLines,
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
pub struct ActionProcessContext {
    pub frontend: FrontendType,
    pub last_action: Option<Action>,
    pub action: Action,
}

#[derive(Debug, Serialize)]
pub struct ActionProcessContextRef<'a> {
    pub frontend: FrontendType,
    pub last_action: Option<&'a Action>,
    pub action: &'a Action,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextProcessContext {
    pub root_path: PathBuf,
    pub game_props: HashMap<String, String>,
    pub frontend: FrontendType,
}

#[derive(Debug, Serialize)]
pub struct TextProcessContextRef<'a> {
    pub root_path: &'a Path,
    pub game_props: &'a HashMap<String, String>,
    pub frontend: FrontendType,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TextProcessResult {
    pub line: ActionLines,
    pub props: HashMap<String, String>,
}
