use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    /// The line plugin.
    /// The custom line types are dealt with this type of plugin.
    pub line: Vec<String>,
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

    /// A line plugins, which provides custom line types.
    pub fn line(mut self, cmds: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.data.line = cmds.into_iter().map(|s| s.into()).collect();
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

/// The argument to action plugin.
///
/// Every action plugin should implement `process_action`:
/// ```ignore
/// use ayaka_bindings::*;
///
/// #[export]
/// fn process_action(mut ctx: ActionProcessContext) -> ActionProcessResult {
///     // Process the action...
///     ActionProcessResult { action: ctx.action }
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct ActionProcessContext {
    /// The global properties of the game profile.
    pub game_props: HashMap<String, String>,
    /// The frontend type.
    pub frontend: FrontendType,
    /// The current context.
    pub ctx: RawContext,
    /// The current action.
    pub action: ActionText,
}

#[derive(Debug, Serialize)]
#[doc(hidden)]
pub struct ActionProcessContextRef<'a> {
    pub game_props: &'a HashMap<String, String>,
    pub frontend: FrontendType,
    pub ctx: &'a RawContext,
    pub action: &'a ActionText,
}

/// The result of action plugins.
/// See examples at [`ActionProcessContext`].
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ActionProcessResult {
    /// The processed action text.
    pub action: ActionText,
}

/// The argument to text plugin.
///
/// ```ignore
/// use ayaka_bindings::*;
///
/// #[export]
/// fn plugin_type() -> PluginType {
///     PluginType::builder().text(&["hello"]).build()
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
    /// The global properties of the game profile.
    pub game_props: HashMap<String, String>,
    /// The frontend type.
    pub frontend: FrontendType,
}

#[derive(Debug, Serialize)]
#[doc(hidden)]
pub struct TextProcessContextRef<'a> {
    pub game_props: &'a HashMap<String, String>,
    pub frontend: FrontendType,
}

/// The result of commands in text plugins.
/// See examples at [`TextProcessContext`].
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TextProcessResult {
    /// The lines to append.
    pub text: ActionText,
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
    /// The global properties of the game.
    pub props: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
#[doc(hidden)]
pub struct GameProcessContextRef<'a> {
    pub title: &'a str,
    pub author: &'a str,
    pub props: &'a HashMap<String, String>,
}

/// The result of game plugins.
/// See examples at [`GameProcessContext`].
#[derive(Debug, Serialize, Deserialize)]
pub struct GameProcessResult {
    /// The updated properties.
    pub props: HashMap<String, String>,
}

/// The argument to line plugin.
///
/// ```ignore
/// use ayaka_bindings::*;
///
/// #[export]
/// fn plugin_type() -> PluginType {
///     PluginType::builder().line(&["hello"]).build()
/// }
///
/// #[export]
/// fn hello(_ctx: LineProcessContext) -> LineProcessResult {
///     let mut res = LineProcessResult::default();
///     res.locals.insert("hello".to_string(), RawValue::Str("world".to_string()));
///     res
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct LineProcessContext {
    /// The global properties of the game profile.
    pub game_props: HashMap<String, String>,
    /// The frontend type.
    pub frontend: FrontendType,
    /// The current context.
    pub ctx: RawContext,
    /// The full properties of the custom command.
    pub props: VarMap,
}

#[derive(Debug, Serialize)]
#[doc(hidden)]
pub struct LineProcessContextRef<'a> {
    pub game_props: &'a HashMap<String, String>,
    pub frontend: FrontendType,
    pub ctx: &'a RawContext,
    pub props: &'a VarMap,
}

/// The result of commands in line plugins.
/// See examples at [`LineProcessContext`].
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LineProcessResult {
    /// The updated variables.
    pub locals: VarMap,
    /// The temp variables.
    pub vars: VarMap,
}
