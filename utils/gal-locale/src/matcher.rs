use icu_locid::LanguageIdentifier;
use icu_locid_transform::LocaleExpander;
use icu_provider_blob::StaticDataProvider;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq)]
enum MatchTag {
    Str(String),
    Var(String),
    VarExclude(String),
    All,
}

impl From<&'_ str> for MatchTag {
    fn from(s: &'_ str) -> Self {
        if s == "*" {
            Self::All
        } else if s.starts_with("$!") {
            Self::VarExclude(s[2..].to_string())
        } else if s.starts_with('$') {
            Self::Var(s[1..].to_string())
        } else {
            Self::Str(s.to_string())
        }
    }
}

impl MatchTag {
    pub fn matches(&self, tag: &str, vars: &Variables) -> bool {
        match self {
            Self::Str(s) => s == tag,
            Self::Var(key) => vars[key].contains(tag),
            Self::VarExclude(key) => !vars[key].contains(tag),
            Self::All => true,
        }
    }

    pub fn option_matches(s: Option<&Self>, tag: Option<&str>, vars: &Variables) -> bool {
        match (s, tag) {
            (None, None) => true,
            (Some(Self::All), _) => true,
            (Some(s), Some(tag)) => s.matches(tag, vars),
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(from = "String")]
struct MatchLanguage {
    pub language: MatchTag,
    pub script: Option<MatchTag>,
    pub region: Option<MatchTag>,
}

impl From<&'_ str> for MatchLanguage {
    fn from(s: &'_ str) -> Self {
        let mut parts = s.split('_');
        let language = parts.next().unwrap().into();
        let script = parts.next().map(|s| s.into());
        let region = parts.next().map(|s| s.into());
        Self {
            language,
            script,
            region,
        }
    }
}

impl From<String> for MatchLanguage {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

impl MatchLanguage {
    pub fn matches(&self, lang: &LanguageIdentifier, vars: &Variables) -> bool {
        self.language.matches(lang.language.as_str(), vars)
            && MatchTag::option_matches(
                self.script.as_ref(),
                lang.script.as_ref().map(|s| s.as_str()),
                vars,
            )
            && MatchTag::option_matches(
                self.region.as_ref(),
                lang.region.as_ref().map(|s| s.as_str()),
                vars,
            )
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
enum LanguageMatchItem {
    ParadigmLocales(ParadigmLocales),
    MatchVariable(MatchVariable),
    LanguageMatch(LanguageMatch),
}

#[derive(Debug, Deserialize, PartialEq)]
struct ParadigmLocales {
    pub locales: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct MatchVariable {
    pub id: String,
    pub value: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct LanguageMatch {
    pub desired: MatchLanguage,
    pub supported: MatchLanguage,
    pub distance: u16,
    #[serde(default)]
    pub oneway: bool,
}

#[derive(Debug, Deserialize, PartialEq)]
struct LanguageMatches {
    #[serde(rename = "$value")]
    pub matches: Vec<LanguageMatchItem>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct LanguageMatching {
    #[serde(rename = "$value")]
    pub matches: LanguageMatches,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct SupplementalData {
    pub language_matching: LanguageMatching,
}

const LANGUAGE_INFO: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/languageInfo.xml"
));
const CLDR_BIN: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/cldr.bin"));

pub struct LanguageMatcher {
    paradiam: Vec<LanguageIdentifier>,
    vars: Variables,
    rules: Vec<LanguageMatch>,
    expander: LocaleExpander,
}

type Variables = HashMap<String, HashSet<String>>;

impl From<SupplementalData> for LanguageMatcher {
    fn from(data: SupplementalData) -> Self {
        let mut paradiam = vec![];
        let mut vars = HashMap::new();
        let mut rules = vec![];
        let data = data.language_matching.matches.matches;
        for item in data {
            match item {
                LanguageMatchItem::ParadigmLocales(ParadigmLocales { locales }) => {
                    let mut locales = locales
                        .split(' ')
                        .map(|s| s.parse().unwrap())
                        .collect::<Vec<_>>();
                    paradiam.append(&mut locales);
                }
                LanguageMatchItem::MatchVariable(MatchVariable { id, value }) => {
                    assert!(id.starts_with('$'));
                    // TODO: we need to support '-' as well, but there's no '-' in the data.
                    vars.insert(
                        id[1..].to_string(),
                        value.split('+').map(|s| s.to_string()).collect(),
                    );
                }
                LanguageMatchItem::LanguageMatch(m) => {
                    rules.push(m);
                }
            }
        }
        let provider = StaticDataProvider::try_new_from_static_blob(CLDR_BIN).unwrap();
        let expander = LocaleExpander::try_new_with_buffer_provider(&provider).unwrap();
        Self {
            paradiam,
            vars,
            rules,
            expander,
        }
    }
}

impl LanguageMatcher {
    pub fn new() -> Self {
        let data: SupplementalData = serde_xml_rs::from_str(LANGUAGE_INFO).unwrap();
        data.into()
    }

    pub fn matches(
        &self,
        desired: LanguageIdentifier,
        supported: impl IntoIterator<Item = LanguageIdentifier>,
    ) -> Option<(LanguageIdentifier, u16)> {
        supported
            .into_iter()
            .map(|s| (s.clone(), self.distance(desired.clone(), s)))
            .min_by_key(|(_, dis)| *dis)
            .filter(|(_, dis)| *dis < 1000)
    }

    pub fn distance(
        &self,
        mut desired: LanguageIdentifier,
        mut supported: LanguageIdentifier,
    ) -> u16 {
        self.expander.maximize(&mut desired);
        self.expander.maximize(&mut supported);

        let mut distance = 0;

        if desired.region != supported.region {
            distance += self.distance_impl(&desired, &supported);
        }
        desired.region = None;
        supported.region = None;

        if desired.script != supported.script {
            distance += self.distance_impl(&desired, &supported);
        }
        desired.script = None;
        supported.script = None;

        if desired.language != supported.language {
            distance += self.distance_impl(&desired, &supported);
        }

        distance
    }

    fn distance_impl(&self, desired: &LanguageIdentifier, supported: &LanguageIdentifier) -> u16 {
        for rule in &self.rules {
            let mut matches = rule.desired.matches(desired, &self.vars)
                && rule.supported.matches(supported, &self.vars);
            if !rule.oneway && !matches {
                matches = rule.supported.matches(desired, &self.vars)
                    && rule.desired.matches(supported, &self.vars);
            }
            if matches {
                let mut distance = rule.distance * 10;
                match (self.is_paradiam(desired), self.is_paradiam(supported)) {
                    (true, false) | (false, true) => distance -= 1,
                    _ => {}
                }
                return distance;
            }
        }
        unreachable!()
    }

    fn is_paradiam(&self, lang: &LanguageIdentifier) -> bool {
        self.paradiam.iter().find(|p| p == &lang).is_some()
    }
}
