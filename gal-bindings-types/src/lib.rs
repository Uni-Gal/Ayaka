use serde::{Deserialize, Serialize};

#[doc(hidden)]
#[derive(Serialize, Deserialize)]
pub struct Record {
    pub level: usize,
    pub target: String,
    pub msg: String,
    pub module_path: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
}

impl From<&log::Record<'_>> for Record {
    fn from(r: &log::Record) -> Self {
        Self {
            level: r.level() as usize,
            target: r.target().to_string(),
            msg: format!("{}", r.args()),
            module_path: r.module_path().map(|s| s.to_string()),
            file: r.file().map(|s| s.to_string()),
            line: r.line(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum PluginType {
    Script,
    Action,
}

#[derive(Serialize, Deserialize)]
pub enum FrontendType {
    Text,
    Html,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ActionData {
    pub line: String,
    pub character: Option<String>,
    pub switches: Vec<Switch>,
    pub bg: Option<String>,
    pub bgm: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Switch {
    pub text: String,
    pub enabled: bool,
}
