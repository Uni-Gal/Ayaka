//! The types used in both runtime and plugins.

#![warn(missing_docs)]
#![deny(unsafe_code)]

use ayaka_script::RawValue;
use fallback::FallbackSpec;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
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

/// The bit flags to describe plugin type.
///
/// Every plugin should provide a function `plugin_type`,
/// which returns [`PluginType`].
///
/// ```ignore
/// use ayaka_bindings::*;
///
/// #[export]
/// fn plugin_type() -> PluginType {
///     PluginType::default()
/// }
/// ```
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PluginType {
    /// The action plugin.
    /// This plugin processes the action after they are parsed.
    pub action: bool,
    /// The text plugin.
    /// The custom text commands are dealt with this type of plugin.
    pub text: Vec<String>,
    /// The game plugin.
    /// This plugin processes the game properties after it is loaded.
    pub game: bool,
}

impl PluginType {
    /// Creates a [`PluginTypeBuilder`] instance to build a [`PluginType`].
    pub fn builder() -> PluginTypeBuilder {
        PluginTypeBuilder {
            data: Self::default(),
        }
    }
}

/// The builder of [`PluginType`].
pub struct PluginTypeBuilder {
    data: PluginType,
}

impl PluginTypeBuilder {
    /// An action plugin.
    pub fn action(mut self) -> Self {
        self.data.action = true;
        self
    }

    /// A text plugin, which provides commands.
    pub fn text(mut self, cmds: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.data.text = cmds.into_iter().map(|s| s.into()).collect();
        self
    }

    /// A game plugin.
    pub fn game(mut self) -> Self {
        self.data.game = true;
        self
    }

    /// Build a [`PluginType`].
    pub fn build(self) -> PluginType {
        self.data
    }
}

/// The type of current frontend.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FrontendType {
    /// The frontend only accepts raw texts.
    Text,
    /// The frontend renders HTML.
    Html,
    /// The frontend renders LaTeX.
    Latex,
}

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

/// The full action information in one line of config.
/// It provides the full texts and other properties exacted from [`ayaka_script::Text`].
///
/// The `text` is a [`VecDeque<ActionSubText>`].
/// The [`ActionSubText`] could be pushed and poped at front or back.
///
/// Generally, you should avoid using `push_back` directly.
/// To reduce allocations in serialization, you should use
/// `push_back_chars` and `push_back_block`.
///
/// ```
/// # use ayaka_bindings_types::*;
/// let mut lines = Action::default();
/// lines.push_back_chars("Hello ");
/// assert_eq!(lines[0], ActionSubText::chars("Hello "));
/// lines.push_back_chars("world!");
/// assert_eq!(lines[0], ActionSubText::chars("Hello world!"));
/// ```
#[derive(Debug, Default, Clone, Serialize, Deserialize, FallbackSpec)]
pub struct ActionText {
    /// The full texts.
    pub text: VecDeque<ActionSubText>,
    /// The key of current character.
    pub ch_key: Option<String>,
    /// The current character.
    pub character: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Text(ActionText),
    Switches(Vec<Switch>),
}

impl Default for Action {
    fn default() -> Self {
        Self::Text(ActionText::default())
    }
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
/// use ayaka_bindings::*;
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
    /// The current action.
    pub action: ActionText,
}

#[derive(Debug, Serialize)]
#[doc(hidden)]
pub struct ActionProcessContextRef<'a> {
    pub root_path: &'a Path,
    pub game_props: &'a HashMap<String, String>,
    pub frontend: FrontendType,
    pub action: &'a ActionText,
}

/// The argument to text plugin.
///
/// Every text plugin should implement `text_commands` and the specified function:
/// ```ignore
/// use ayaka_bindings::*;
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
    pub text: VecDeque<ActionSubText>,
}

/// The argument to game plugin.
///
/// Every game plugin should implement `process_game`:
/// ```ignore
/// use ayaka_bindings::*;
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
