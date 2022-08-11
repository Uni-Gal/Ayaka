//! The types used in both runtime and plugins.

#![warn(missing_docs)]
#![deny(unsafe_code)]

use gal_fallback::{FallbackSpec, IsEmpty2};
use gal_script::{Program, RawValue};
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
    /// The bit flags to describe plugin type.
    ///
    /// Every plugin should provide a function `plugin_type`,
    /// which returns [`PluginType`].
    ///
    /// ```ignore
    /// use gal_bindings::*;
    ///
    /// #[export]
    /// fn plugin_type() -> PluginType {
    ///     PluginType::SCRIPT
    /// }
    /// ```
    #[derive(Serialize, Deserialize)]
    pub struct PluginType: u32 {
        /// The default value.
        /// All plugins are script plugins.
        const SCRIPT = 0b000;
        /// The action plugin.
        /// This plugin processes the action after they are parsed.
        const ACTION = 0b001;
        /// The text plugin.
        /// The custom text commands are dealt with this type of plugin.
        const TEXT   = 0b010;
        /// The game plugin.
        /// This plugin processes the game properties after it is loaded.
        const GAME   = 0b100;
    }
}

/// The type of current frontend.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FrontendType {
    /// The frontend only accepts raw texts.
    Text,
    /// The frontend renders HTML.
    Html,
}

/// The unit of one line in an action.
///
/// If a frontend supports animation,
/// the characters in [`ActionLine::Chars`] should be printed one by one,
/// while the characters in [`ActionLine::Block`] should be printed together.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ActionLine {
    /// Characters printed one by one.
    /// Usually they are meaningful texts.
    Chars(String),
    /// Characters printed together.
    /// Usually they are HTML tags or other control characters.
    Block(String),
}

impl ActionLine {
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

/// A collection of [`ActionLine`].
///
/// Internally it is a [`VecDeque<ActionLine>`].
/// The [`ActionLine`] could be pushed and poped at front or back.
///
/// Generally, you should avoid using `push_back` directly.
/// To reduce allocations in serialization, you should use
/// `push_back_chars` and `push_back_block`.
///
/// ```
/// # use gal_bindings_types::*;
/// let mut lines = ActionLines::default();
/// lines.push_back_chars("Hello ");
/// assert_eq!(lines[0], ActionLine::chars("Hello "));
/// lines.push_back_chars("world!");
/// assert_eq!(lines[0], ActionLine::chars("Hello world!"));
/// ```
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
    /// Push the string as [`ActionLine::Chars`] to the back.
    /// If the back element is also [`ActionLine::Chars`], the string is appended.
    pub fn push_back_chars<'a>(&mut self, s: impl Into<Cow<'a, str>>) {
        let s = s.into();
        if let Some(ActionLine::Chars(text)) = self.back_mut() {
            text.push_str(&s);
        } else {
            self.push_back(ActionLine::chars(s));
        }
    }

    /// Push the string as [`ActionLine::Block`] to the back.
    /// If the back element is also [`ActionLine::Block`], the string is appended.
    pub fn push_back_block<'a>(&mut self, s: impl Into<Cow<'a, str>>) {
        let s = s.into();
        if let Some(ActionLine::Block(text)) = self.back_mut() {
            text.push_str(&s);
        } else {
            self.push_back(ActionLine::block(s));
        }
    }
}

impl IsEmpty2 for ActionLines {
    fn is_empty2(&self) -> bool {
        self.0.is_empty()
    }
}

/// A map from variable name to [`RawValue`].
pub type VarMap = HashMap<String, RawValue>;

/// The serializable context.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct RawContext {
    /// Current paragraph tag.
    pub cur_para: String,
    /// Current text index.
    pub cur_act: usize,
    /// Current local variables.
    pub locals: VarMap,
}

/// The full action information in one line of config.
/// It provides the full texts and other properties exacted from [`gal_script::Text`].
#[derive(Debug, Default, Clone, Serialize, Deserialize, FallbackSpec)]
pub struct Action {
    /// The context snapshot.
    pub ctx: RawContext,
    /// The full texts.
    pub line: ActionLines,
    /// The current character.
    pub character: Option<String>,
    /// The title of current paragraph.
    pub para_title: Option<String>,
    /// The switches.
    pub switches: Vec<Switch>,
    /// The actions of switches.
    pub switch_actions: Vec<Program>,
    /// The other custom properties.
    pub props: HashMap<String, String>,
}

/// One switch in the switches of an [`Action`].
#[derive(Debug, Default, Clone, Serialize, Deserialize, FallbackSpec)]
pub struct Switch {
    /// The switch text.
    pub text: String,
    /// Whether the switch is enabled.
    pub enabled: bool,
}

/// The argument to action plugin.
///
/// Every action plugin should implement `process_action`:
/// ```ignore
/// use gal_bindings::*;
///
/// #[export]
/// fn process_action(mut ctx: ActionProcessContext) -> Action {
///     // Process the action...
///     ctx.action
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct ActionProcessContext {
    /// The root path of the game profile.
    pub root_path: PathBuf,
    /// The global properties of the game profile.
    pub game_props: HashMap<String, String>,
    /// The frontend type.
    pub frontend: FrontendType,
    /// The previous action in the history.
    /// It is used if some properties need to inherit.
    pub last_action: Option<Action>,
    /// The current action.
    pub action: Action,
}

#[derive(Debug, Serialize)]
#[doc(hidden)]
pub struct ActionProcessContextRef<'a> {
    pub root_path: &'a Path,
    pub game_props: &'a HashMap<String, String>,
    pub frontend: FrontendType,
    pub last_action: Option<&'a Action>,
    pub action: &'a Action,
}

/// The argument to text plugin.
///
/// Every text plugin should implement `text_commands` and the specified function:
/// ```ignore
/// use gal_bindings::*;
///
/// #[export]
/// fn text_commands() -> &'static [&'static str] {
///     &["hello"]
/// }
///
/// #[export]
/// fn hello(_args: Vec<String>, _ctx: TextProcessContext) -> TextProcessResult {
///     let mut res = TextProcessResult::default();
///     res.line.push_back_chars("hello");
///     res
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct TextProcessContext {
    /// The root path of the game profile.
    pub root_path: PathBuf,
    /// The global properties of the game profile.
    pub game_props: HashMap<String, String>,
    /// The frontend type.
    pub frontend: FrontendType,
}

#[derive(Debug, Serialize)]
#[doc(hidden)]
pub struct TextProcessContextRef<'a> {
    pub root_path: &'a Path,
    pub game_props: &'a HashMap<String, String>,
    pub frontend: FrontendType,
}

/// The result of commands in text plugins.
/// See examples at [`TextProcessContext`].
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TextProcessResult {
    /// The lines to append.
    pub line: ActionLines,
    /// The custom properties to update.
    pub props: HashMap<String, String>,
}

/// The argument to game plugin.
///
/// Every game plugin should implement `process_game`:
/// ```ignore
/// use gal_bindings::*;
///
/// #[export]
/// fn process_game(mut ctx: GameProcessContext) -> GameProcessResult {
///     // Process the game...
///     GameProcessResult { props: ctx.props }
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct GameProcessContext {
    /// The title of the game.
    pub title: String,
    /// The author of the game.
    pub author: String,
    /// The root path of the game profile.
    pub root_path: PathBuf,
    /// The global properties of the game.
    pub props: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
#[doc(hidden)]
pub struct GameProcessContextRef<'a> {
    pub title: &'a str,
    pub author: &'a str,
    pub root_path: &'a Path,
    pub props: &'a HashMap<String, String>,
}

/// The result of game plugins.
/// See examples at [`GameProcessContext`].
#[derive(Debug, Serialize, Deserialize)]
pub struct GameProcessResult {
    /// The updated properties.
    pub props: HashMap<String, String>,
}
