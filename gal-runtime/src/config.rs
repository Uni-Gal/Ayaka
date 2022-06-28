use gal_locale::Locale;
use gal_script::{
    log::{trace, warn},
    RawValue,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
pub struct Game {
    #[serde(skip)]
    pub root_path: PathBuf,
    pub title: String,
    #[serde(default)]
    pub author: String,
    pub paras: HashMap<Locale, Vec<Paragraph>>,
    #[serde(default)]
    pub plugins: PathBuf,
    #[serde(default)]
    pub res: HashMap<Locale, VarMap>,
    pub base_lang: Locale,
}

impl Game {
    fn choose_from_keys<V>(&self, loc: &Locale, map: &HashMap<Locale, V>) -> Locale {
        let keys = map.keys();
        trace!("Choose \"{}\" from {:?}", loc, keys);
        let res = loc
            .choose_from(keys)
            .unwrap_or_else(|e| {
                warn!("Cannot choose locale: {}", e);
                None
            })
            .unwrap_or_else(|| self.base_lang.clone());
        trace!("Chose \"{}\"", res);
        res
    }

    pub fn find_para(&self, loc: &Locale, tag: &str) -> Option<&Paragraph> {
        if let Some(paras) = self.paras.get(&self.choose_from_keys(loc, &self.paras)) {
            for p in paras {
                if p.tag == tag {
                    return Some(p);
                }
            }
        }
        None
    }

    pub fn find_para_fallback(&self, loc: &Locale, tag: &str) -> Fallback<Paragraph> {
        Fallback::new(
            self.find_para(loc, tag),
            self.find_para(&self.base_lang, tag),
        )
    }

    pub fn find_res(&self, loc: &Locale) -> Option<&HashMap<String, RawValue>> {
        self.res.get(&self.choose_from_keys(loc, &self.res))
    }

    pub fn find_res_fallback(&self, loc: &Locale) -> Fallback<HashMap<String, RawValue>> {
        Fallback::new(self.find_res(loc), self.find_res(&self.base_lang))
    }
}

#[derive(Debug, Deserialize)]
pub struct Paragraph {
    pub tag: String,
    pub title: Option<String>,
    pub texts: Vec<String>,
    pub next: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct RawContext {
    pub cur_para: String,
    pub cur_act: usize,
    pub locals: VarMap,
}

pub type VarMap = HashMap<String, RawValue>;

pub struct Fallback<'a, T> {
    data: Option<&'a T>,
    base_data: Option<&'a T>,
}

impl<'a, T> Fallback<'a, T> {
    pub(crate) fn new(data: Option<&'a T>, base_data: Option<&'a T>) -> Self {
        Self { data, base_data }
    }

    pub fn is_some(&self) -> bool {
        self.data.is_some() || self.base_data.is_some()
    }

    pub fn and_then<V>(&self, mut f: impl FnMut(&'a T) -> Option<V>) -> Option<V> {
        self.data.and_then(|t| f(t)).or_else(|| {
            trace!("Fallback occurred");
            self.base_data.and_then(|t| f(t))
        })
    }
}
