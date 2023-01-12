use ayaka_primitive::RawValue;
use fallback::FallbackSpec;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
};

/// The unit of one line in an action.
///
/// If a frontend supports animation,
/// the characters in [`ActionLine::Chars`] should be printed one by one,
/// while the characters in [`ActionLine::Block`] should be printed together.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ActionSubText {
    /// Characters printed one by one.
    /// Usually they are meaningful texts.
    Chars(String),
    /// Characters printed together.
    /// Usually they are HTML tags or other control characters.
    Block(String),
}

impl ActionSubText {
    /// Creates [`ActionLine::Chars`].
    pub fn chars(s: impl Into<String>) -> Self {
        Self::Chars(s.into())
    }

    /// Creates [`ActionLine::Block`].
    pub fn block(s: impl Into<String>) -> Self {
        Self::Block(s.into())
    }

    /// Gets a reference of [`str`].
    pub fn as_str(&self) -> &str {
        match self {
            Self::Chars(s) | Self::Block(s) => s,
        }
    }

    /// Gets the inner [`String`].
    pub fn into_string(self) -> String {
        match self {
            Self::Chars(s) | Self::Block(s) => s,
        }
    }
}

/// A map from variable name to [`RawValue`].
pub type VarMap = HashMap<String, RawValue>;

/// The serializable context.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct RawContext {
    /// Current base paragraph tag.
    pub cur_base_para: String,
    /// Current paragraph tag.
    pub cur_para: String,
    /// Current text index.
    pub cur_act: usize,
    /// Current local variables.
    pub locals: VarMap,
}

/// The `text` is a [`VecDeque<ActionSubText>`].
/// The [`ActionSubText`] could be pushed and poped at front or back.
///
/// Generally, you should avoid using `push_back` directly.
/// To reduce allocations in serialization, you should use
/// `push_back_chars` and `push_back_block`.
///
/// ```
/// # use ayaka_bindings_types::*;
/// let mut text = ActionText::default();
/// text.push_back_chars("Hello ");
/// assert_eq!(text.text[0], ActionSubText::chars("Hello "));
/// text.push_back_chars("world!");
/// assert_eq!(text.text[0], ActionSubText::chars("Hello world!"));
/// ```
#[derive(Debug, Default, Clone, Serialize, Deserialize, FallbackSpec)]
pub struct ActionText {
    /// The full texts.
    pub text: VecDeque<ActionSubText>,
    /// The key of current character.
    pub ch_key: Option<String>,
    /// The current character.
    pub character: Option<String>,
    /// The temp variables.
    pub vars: VarMap,
}

impl ActionText {
    /// Push the string as [`ActionLine::Chars`] to the back.
    /// If the back element is also [`ActionLine::Chars`], the string is appended.
    pub fn push_back_chars<'a>(&mut self, s: impl Into<Cow<'a, str>>) {
        let s = s.into();
        if let Some(ActionSubText::Chars(text)) = self.text.back_mut() {
            text.push_str(&s);
        } else {
            self.text.push_back(ActionSubText::chars(s));
        }
    }

    /// Push the string as [`ActionLine::Block`] to the back.
    /// If the back element is also [`ActionLine::Block`], the string is appended.
    pub fn push_back_block<'a>(&mut self, s: impl Into<Cow<'a, str>>) {
        let s = s.into();
        if let Some(ActionSubText::Block(text)) = self.text.back_mut() {
            text.push_str(&s);
        } else {
            self.text.push_back(ActionSubText::block(s));
        }
    }
}

impl std::fmt::Display for ActionText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for text in &self.text {
            write!(f, "{}", text.as_str())?;
        }
        Ok(())
    }
}

/// The full action information in one line of config.
/// It provides the full texts and other properties exacted from [`ayaka_script::Text`].
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Action {
    /// An empty action usually means an `exec` or custom action.
    #[default]
    Empty,
    /// A text action, display some texts.
    Text(ActionText),
    /// A switch action, display switches and let player to choose.
    Switches(Vec<Switch>),
    /// A custom action.
    Custom(VarMap),
}

/// One switch in the switches of an [`Action`].
#[derive(Debug, Default, Clone, Serialize, Deserialize, FallbackSpec)]
pub struct Switch {
    /// The switch text.
    pub text: String,
    /// Whether the switch is enabled.
    pub enabled: bool,
}
