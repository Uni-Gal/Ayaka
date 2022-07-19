pub use gal_bindings_types::{Action, Switch};
pub use gal_fallback::Fallback;

use crate::*;
use gal_script::log::{debug, warn};
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Deserialize)]
pub struct Paragraph {
    pub tag: String,
    pub title: Option<String>,
    pub texts: Vec<String>,
    pub next: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Game {
    pub title: String,
    #[serde(default)]
    pub author: String,
    pub paras: HashMap<LocaleBuf, Vec<Paragraph>>,
    #[serde(default)]
    pub plugins: PluginsConfig,
    #[serde(default)]
    pub props: HashMap<String, String>,
    #[serde(default)]
    pub res: HashMap<LocaleBuf, VarMap>,
    pub base_lang: LocaleBuf,
}

#[derive(Debug, Default, Deserialize)]
pub struct PluginsConfig {
    pub dir: PathBuf,
    #[serde(default)]
    pub modules: Vec<String>,
}

impl Game {
    fn choose_from_keys<V>(&self, loc: &Locale, map: &HashMap<LocaleBuf, V>) -> LocaleBuf {
        let keys = map.keys();
        debug!("Choose \"{}\" from {:?}", loc, keys);
        let res = loc
            .choose_from(keys)
            .unwrap_or_else(|e| {
                warn!("Cannot choose locale: {}", e);
                None
            })
            .unwrap_or_else(|| self.base_lang.clone());
        debug!("Chose \"{}\"", res);
        res
    }

    fn find_para(&self, loc: &Locale, tag: &str) -> Option<&Paragraph> {
        if let Some(paras) = self.paras.get(loc) {
            for p in paras {
                if p.tag == tag {
                    return Some(p);
                }
            }
        }
        None
    }

    pub fn find_para_fallback(&self, loc: &Locale, tag: &str) -> Fallback<&Paragraph> {
        let key = self.choose_from_keys(loc, &self.paras);
        let base_key = self.choose_from_keys(&self.base_lang, &self.paras);
        Fallback::new(
            if key == base_key {
                None
            } else {
                self.find_para(&key, tag)
            },
            self.find_para(&base_key, tag),
        )
    }

    fn find_res(&self, loc: &Locale) -> Option<&VarMap> {
        self.res.get(loc)
    }

    pub fn find_res_fallback(&self, loc: &Locale) -> Fallback<&VarMap> {
        let key = self.choose_from_keys(loc, &self.res);
        let base_key = self.choose_from_keys(&self.base_lang, &self.res);
        Fallback::new(
            if key == base_key {
                None
            } else {
                self.find_res(&key)
            },
            self.find_res(&base_key),
        )
    }
}
