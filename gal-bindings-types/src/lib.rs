use serde::{Deserialize, Serialize};

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
