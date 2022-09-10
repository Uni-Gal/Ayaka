use crate::*;

/// A collection of [`Line`].
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Text(pub Vec<Line>);

/// A part of a line, either some texts or a command.
#[derive(Debug, PartialEq, Eq)]
pub enum Line {
    /// Raw texts.
    Str(String),
    /// A command. See [`Command`].
    Cmd(Command),
}

/// A TeX-like command in the text.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// `\ch{}{}`
    ///
    /// Controls the current character.
    Character(String, String),
    /// `\exec{}`
    ///
    /// Executes a program and calculates the return value into text.
    Exec(Program),
    /// `\switch{}{}{}`
    ///
    /// A switch.
    Switch {
        /// The text of the switch.
        text: String,
        /// The action after choosing the switch,
        action: Program,
        /// The expression determines whether the switch is enabled.
        enabled: Option<Program>,
    },
    /// Other custom commands.
    Other(String, Vec<String>),
}
