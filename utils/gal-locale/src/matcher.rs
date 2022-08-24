use icu_locid::LanguageIdentifier;
use icu_locid_transform::LocaleExpander;
use icu_provider_blob::StaticDataProvider;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

trait Rule<T> {
    fn matches(self, tag: T, vars: &Variables) -> bool;
}

#[derive(Debug, PartialEq)]
enum SubTagRule {
    Str(String),
    Var(String),
    VarExclude(String),
    All,
}

impl From<&'_ str> for SubTagRule {
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

impl Rule<&'_ str> for &'_ SubTagRule {
    fn matches(self, tag: &str, vars: &Variables) -> bool {
        match self {
            SubTagRule::Str(s) => s == tag,
            SubTagRule::Var(key) => vars[key].contains(tag),
            SubTagRule::VarExclude(key) => !vars[key].contains(tag),
            SubTagRule::All => true,
        }
    }
}

impl Rule<Option<&'_ str>> for Option<&'_ SubTagRule> {
    fn matches(self, tag: Option<&str>, vars: &Variables) -> bool {
        match (self, tag) {
            (None, None) | (Some(SubTagRule::All), _) => true,
            (Some(s), Some(tag)) => s.matches(tag, vars),
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(from = "String")]
struct LanguageIdentifierRule {
    pub language: SubTagRule,
    pub script: Option<SubTagRule>,
    pub region: Option<SubTagRule>,
}

impl From<&'_ str> for LanguageIdentifierRule {
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

impl From<String> for LanguageIdentifierRule {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

impl Rule<&'_ LanguageIdentifier> for &'_ LanguageIdentifierRule {
    fn matches(self, lang: &LanguageIdentifier, vars: &Variables) -> bool {
        self.language.matches(lang.language.as_str(), vars)
            && self
                .script
                .as_ref()
                .matches(lang.script.as_ref().map(|s| s.as_str()), vars)
            && self
                .region
                .as_ref()
                .matches(lang.region.as_ref().map(|s| s.as_str()), vars)
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
    pub desired: LanguageIdentifierRule,
    pub supported: LanguageIdentifierRule,
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
    pub language_matches: LanguageMatches,
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
    paradiam: HashSet<LanguageIdentifier>,
    vars: Variables,
    rules: Vec<LanguageMatch>,
    expander: LocaleExpander,
}

type Variables = HashMap<String, HashSet<String>>;

impl From<SupplementalData> for LanguageMatcher {
    fn from(data: SupplementalData) -> Self {
        let mut paradiam = HashSet::new();
        let mut vars = HashMap::new();
        let mut rules = vec![];
        let data = data.language_matching.language_matches.matches;
        for item in data {
            match item {
                LanguageMatchItem::ParadigmLocales(ParadigmLocales { locales }) => {
                    let locales = locales.split(' ').map(|s| s.parse().unwrap());
                    paradiam.extend(locales)
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
        mut desired: LanguageIdentifier,
        supported: impl IntoIterator<Item = LanguageIdentifier>,
    ) -> Option<(LanguageIdentifier, u16)> {
        self.expander.maximize(&mut desired);
        supported
            .into_iter()
            .map(|s| {
                let mut max_s = s.clone();
                self.expander.maximize(&mut max_s);
                (s, self.distance(desired.clone(), max_s))
            })
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
        self.distance_impl(desired, supported)
    }

    fn distance_impl(
        &self,
        mut desired: LanguageIdentifier,
        mut supported: LanguageIdentifier,
    ) -> u16 {
        debug_assert!(desired.region.is_some());
        debug_assert!(desired.script.is_some());
        debug_assert!(supported.region.is_some());
        debug_assert!(supported.script.is_some());

        let mut distance = 0;

        if desired.region != supported.region {
            distance += self.distance_match(&desired, &supported);
        }
        desired.region = None;
        supported.region = None;

        if desired.script != supported.script {
            distance += self.distance_match(&desired, &supported);
        }
        desired.script = None;
        supported.script = None;

        if desired.language != supported.language {
            distance += self.distance_match(&desired, &supported);
        }

        distance
    }

    fn distance_match(&self, desired: &LanguageIdentifier, supported: &LanguageIdentifier) -> u16 {
        for rule in &self.rules {
            let mut matches = rule.desired.matches(desired, &self.vars)
                && rule.supported.matches(supported, &self.vars);
            if !rule.oneway && !matches {
                matches = rule.supported.matches(desired, &self.vars)
                    && rule.desired.matches(supported, &self.vars);
            }
            if matches {
                let mut distance = rule.distance * 10;
                if self.is_paradiam(desired) ^ self.is_paradiam(supported) {
                    distance -= 1
                }
                return distance;
            }
        }
        unreachable!()
    }

    fn is_paradiam(&self, lang: &LanguageIdentifier) -> bool {
        self.paradiam.contains(lang)
    }
}

#[cfg(test)]
mod test {
    use super::LanguageMatcher;
    use icu_locid::langid;

    #[test]
    fn distance() {
        let matcher = LanguageMatcher::new();

        assert_eq!(matcher.distance(langid!("zh-CN"), langid!("zh-Hans")), 0);
        assert_eq!(matcher.distance(langid!("zh-TW"), langid!("zh-Hant")), 0);
        assert_eq!(matcher.distance(langid!("zh-HK"), langid!("zh-MO")), 40);
        assert_eq!(matcher.distance(langid!("zh-HK"), langid!("zh-Hant")), 50);
    }
}
