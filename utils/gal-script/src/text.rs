//! The text parser.

use crate::exec::*;
use regex::Regex;
use std::{error::Error, fmt::Display, iter::Peekable, str::CharIndices, sync::LazyLock};

static SPACE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\s+)").unwrap());

/// The location of a token.
/// The `Loc(start, end)` means the location `[start, end)`.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Loc(pub usize, pub usize);

impl Loc {
    /// Combines a series of [`Loc`].
    ///
    /// ```
    /// # use gal_script::Loc;
    /// let full_loc = Loc::from_locs([Loc(1, 2), Loc(4, 6), Loc(5, 8)]);
    /// assert_eq!(full_loc, Loc(1, 8));
    /// ```
    pub fn from_locs(locs: impl IntoIterator<Item = Loc>) -> Self {
        let mut start = usize::MAX;
        let mut end = 0;
        for loc in locs.into_iter() {
            start = loc.0.min(start);
            end = loc.1.max(end);
        }
        Self(start, end)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Token<'a> {
    loc: Loc,
    tok: TokenType<'a>,
}

#[derive(Debug, PartialEq, Eq)]
enum TokenType<'a> {
    Space,
    SpecChar(char),
    Text(&'a str),
}

impl<'a> Token<'a> {
    pub fn space(loc: Loc) -> Self {
        Self {
            loc,
            tok: TokenType::Space,
        }
    }

    pub fn spec_char(loc: Loc, c: char) -> Self {
        Self {
            loc,
            tok: TokenType::SpecChar(c),
        }
    }

    pub fn text(loc: Loc, s: &'a str) -> Self {
        Self {
            loc,
            tok: TokenType::Text(s),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct RichToken<'a> {
    loc: Loc,
    tok: RichTokenType<'a>,
}

#[derive(Debug, PartialEq, Eq)]
enum RichTokenType<'a> {
    Char(char),
    Text(&'a str),
    Character(&'a str, &'a str),
    Command(&'a str, Vec<Vec<RichToken<'a>>>),
}

impl<'a> RichToken<'a> {
    pub fn char(loc: Loc, c: char) -> Self {
        Self {
            loc,
            tok: RichTokenType::Char(c),
        }
    }

    pub fn text(loc: Loc, s: &'a str) -> Self {
        Self {
            loc,
            tok: RichTokenType::Text(s),
        }
    }

    pub fn character(loc: Loc, s: &'a str, a: &'a str) -> Self {
        Self {
            loc,
            tok: RichTokenType::Character(s, a),
        }
    }

    pub fn command(loc: Loc, n: &'a str, params: Vec<Vec<RichToken<'a>>>) -> Self {
        Self {
            loc,
            tok: RichTokenType::Command(n, params),
        }
    }
}

/// The error when parsing [`Text`].
#[derive(Debug, PartialEq, Eq)]
pub struct ParseError {
    loc: Loc,
    err: ParseErrorType,
}

impl ParseError {
    pub(crate) fn new(loc: Loc, err: ParseErrorType) -> Self {
        Self { loc, err }
    }

    /// The error location.
    pub fn loc(&self) -> Loc {
        self.loc
    }

    /// The error type.
    pub fn error(&self) -> &ParseErrorType {
        &self.err
    }
}

fn parse_error<T>(loc: Loc, err: ParseErrorType) -> ParseResult<T> {
    Err(ParseError::new(loc, err))
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.err.fmt(f)
    }
}

impl Error for ParseError {}

/// The type of [`ParseError`].
#[derive(Debug, PartialEq, Eq)]
pub enum ParseErrorType {
    /// Illegal char.
    /// Usually unexcepted char after `\`,
    /// or redundant `/`.
    IllegalChar(char),
    /// Illegal space.
    /// The name in `\ch` command cannot contain spaces.
    IllegalSpace,
    /// No command name found after `\`.
    CmdNotFound,
    /// We don't support embedded command inside parameters.
    CmdInCmd,
    /// The builtin commands check the parameters count.
    InvalidParamsCount(String, usize),
    /// An error occurred when parsing [`Program`].
    InvalidProgram(String),
}

impl Display for ParseErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IllegalChar(c) => write!(f, "Illegal char \"{}\".", c.escape_default())?,
            Self::IllegalSpace => write!(f, "Illegal space.")?,
            Self::CmdNotFound => write!(f, "Command not found after \"\\\".")?,
            Self::CmdInCmd => write!(f, "Embedded command is not supported.")?,
            Self::InvalidParamsCount(name, count) => write!(
                f,
                "Invalid params count {} for \"{}\"",
                count,
                name.escape_default()
            )?,
            Self::InvalidProgram(err) => write!(f, "Program parse error: {}", err)?,
        }
        Ok(())
    }
}

/// The [`std::result::Result`] when parsing [`Text`].
pub type ParseResult<T> = std::result::Result<T, ParseError>;

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

/// A collection of [`Line`].
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Text(pub Vec<Line>);

const fn is_special_char(c: char) -> bool {
    matches!(c, '\\' | '{' | '}' | '/')
}

struct TextLexer<'a> {
    text: &'a str,
    chars: Peekable<CharIndices<'a>>,
}

impl<'a> TextLexer<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            chars: text.char_indices().peekable(),
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, c)| *c)
    }

    fn next_char(&mut self) -> Option<char> {
        self.chars.next().map(|(_, c)| c)
    }

    fn offset(&mut self) -> usize {
        self.chars
            .peek()
            .map(|(offset, _)| *offset)
            .unwrap_or_else(|| self.text.len())
    }
}

impl<'a> Iterator for TextLexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.offset();
        let mut has_whitespace = false;
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.next_char();
                has_whitespace = true;
            } else {
                break;
            }
        }
        if has_whitespace {
            return Some(Token::space(Loc(cur, self.offset())));
        }
        let cur = self.offset();
        while let Some(c) = self.peek_char() {
            if is_special_char(c) {
                if self.offset() - cur > 0 {
                    break;
                } else {
                    self.next_char();
                    return Some(Token::spec_char(Loc(self.offset() - 1, self.offset()), c));
                }
            } else if c.is_whitespace() {
                if self.offset() - cur > 0 {
                    break;
                } else {
                    return self.next();
                }
            } else {
                self.next_char();
            }
        }
        if self.offset() - cur > 0 {
            Some(Token::text(
                Loc(cur, self.offset()),
                &self.text[cur..self.offset()],
            ))
        } else {
            None
        }
    }
}

struct TextRichLexer<'a> {
    lexer: Peekable<TextLexer<'a>>,
    in_param: usize,
}

impl<'a> TextRichLexer<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            lexer: TextLexer::new(text).peekable(),
            in_param: 0,
        }
    }

    fn parse_spec_char(&mut self, loc: Loc, c: char) -> ParseResult<RichToken<'a>> {
        match c {
            '\\' => self.parse_escape_or_command(loc),
            '{' | '}' if self.in_param > 0 => Ok(RichToken::char(loc, c)),
            '/' => self.parse_character(loc),
            _ => parse_error(loc, ParseErrorType::IllegalChar(c)),
        }
    }

    fn parse_character(&mut self, prev_loc: Loc) -> ParseResult<RichToken<'a>> {
        let (name, mid_loc) = self.parse_character_name(prev_loc)?;
        let (alias, last_loc) = self.parse_character_name(mid_loc)?;
        Ok(RichToken::character(
            Loc::from_locs([prev_loc, last_loc].into_iter()),
            name,
            alias,
        ))
    }

    fn parse_character_name(&mut self, prev_loc: Loc) -> ParseResult<(&'a str, Loc)> {
        let name = if let Some(tok) = self.lexer.next() {
            match tok.tok {
                TokenType::Space => parse_error(tok.loc, ParseErrorType::IllegalSpace),
                TokenType::SpecChar(c) => match c {
                    '/' => Ok(""),
                    _ => parse_error(tok.loc, ParseErrorType::IllegalChar(c)),
                },
                TokenType::Text(name) => Ok(name),
            }
        } else {
            parse_error(prev_loc, ParseErrorType::IllegalChar('/'))
        }?;
        let last_loc = if !name.is_empty() {
            match self.lexer.next() {
                Some(tok) => match tok.tok {
                    TokenType::SpecChar('/') => Ok(tok.loc),
                    _ => parse_error(prev_loc, ParseErrorType::IllegalChar('/')),
                },
                None => parse_error(prev_loc, ParseErrorType::IllegalChar('/')),
            }
        } else {
            Ok(Loc(prev_loc.0 + 1, prev_loc.1 + 1))
        }?;
        Ok((name, last_loc))
    }

    fn parse_escape_or_command(&mut self, prev_loc: Loc) -> ParseResult<RichToken<'a>> {
        if let Some(tok) = self.lexer.next() {
            match tok.tok {
                TokenType::Space => Ok(RichToken::char(tok.loc, ' ')),
                TokenType::SpecChar(c) => Ok(RichToken::char(tok.loc, c)),
                TokenType::Text(name) => {
                    if self.in_param > 0 {
                        parse_error(tok.loc, ParseErrorType::CmdInCmd)
                    } else {
                        self.parse_params(Loc::from_locs([prev_loc, tok.loc].into_iter()), name)
                    }
                }
            }
        } else {
            parse_error(prev_loc, ParseErrorType::CmdNotFound)
        }
    }

    fn parse_params(&mut self, prev_loc: Loc, name: &'a str) -> ParseResult<RichToken<'a>> {
        if let Some(tok) = self.lexer.peek() {
            let loc = tok.loc;
            match &tok.tok {
                TokenType::Space => {
                    self.lexer.next();
                    Ok(RichToken::command(prev_loc, name, vec![]))
                }
                &TokenType::SpecChar(c) => match c {
                    '\\' => Ok(RichToken::command(prev_loc, name, vec![])),
                    '{' => {
                        let mut params = vec![];
                        while let Some(tok) = self.lexer.peek() {
                            if tok.tok == TokenType::SpecChar('{') {
                                self.lexer.next();
                                let param = self.parse_param()?;
                                params.push(param);
                            } else {
                                break;
                            }
                        }
                        Ok(RichToken::command(prev_loc, name, params))
                    }
                    _ => parse_error(loc, ParseErrorType::IllegalChar(c)),
                },
                TokenType::Text(_) => parse_error(loc, ParseErrorType::CmdNotFound),
            }
        } else {
            Ok(RichToken::command(prev_loc, name, vec![]))
        }
    }

    fn parse_param(&mut self) -> ParseResult<Vec<RichToken<'a>>> {
        self.in_param += 1;
        let mut tokens = vec![];
        while let Some(tok) = self.lexer.next() {
            match tok.tok {
                TokenType::Space => tokens.push(RichToken::char(tok.loc, ' ')),
                TokenType::SpecChar(c) => {
                    match c {
                        '{' => self.in_param += 1,
                        '}' => {
                            self.in_param -= 1;
                            if self.in_param == 0 {
                                break;
                            }
                        }
                        _ => {}
                    };
                    tokens.push(self.parse_spec_char(tok.loc, c)?);
                }
                TokenType::Text(s) => tokens.push(RichToken::text(tok.loc, s)),
            }
        }
        Ok(tokens)
    }
}

impl<'a> Iterator for TextRichLexer<'a> {
    type Item = ParseResult<RichToken<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(tok) = self.lexer.next() {
            match tok.tok {
                TokenType::Space => Some(Ok(RichToken::char(tok.loc, ' '))),
                TokenType::SpecChar(c) => Some(self.parse_spec_char(tok.loc, c)),
                TokenType::Text(s) => Some(Ok(RichToken::text(tok.loc, s))),
            }
        } else {
            None
        }
    }
}

/// The parser of [`Text`].
pub struct TextParser<'a> {
    lexer: Peekable<TextRichLexer<'a>>,
}

impl<'a> TextParser<'a> {
    /// Create a new [`TextParser`] from a string.
    pub fn new(text: &'a str) -> Self {
        Self {
            lexer: TextRichLexer::new(text).peekable(),
        }
    }

    /// Parse into [`Text`].
    pub fn parse(mut self) -> ParseResult<Text> {
        Ok(Text(self.try_collect()?))
    }

    fn parse_next(&mut self) -> ParseResult<Option<Line>> {
        let mut str = String::new();
        while let Some(tok) = self.lexer.peek() {
            match tok {
                Ok(tok) => match &tok.tok {
                    RichTokenType::Char(c) => {
                        str.push(*c);
                        self.lexer.next();
                    }
                    RichTokenType::Text(s) => {
                        str.push_str(s);
                        self.lexer.next();
                    }
                    RichTokenType::Character(name, alias) => {
                        if str.is_empty() {
                            let name = name.to_string();
                            let alias = alias.to_string();
                            self.lexer.next();
                            return Ok(Some(Line::Cmd(Command::Character(name, alias))));
                        } else {
                            break;
                        }
                    }
                    RichTokenType::Command(name, params) => {
                        if str.is_empty() {
                            let res = Self::parse_command(tok.loc, name, params)?;
                            self.lexer.next();
                            return Ok(Some(res));
                        } else {
                            break;
                        }
                    }
                },
                Err(_) => {
                    // unwrap: peek succeeded.
                    self.lexer.next().unwrap()?;
                }
            }
        }
        if !str.is_empty() {
            let trimmed_str = SPACE_REGEX.replace_all(&str, " ");
            Ok(Some(Line::Str(trimmed_str.into_owned())))
        } else {
            Ok(None)
        }
    }

    fn concat_params(toks: &[RichToken]) -> ParseResult<String> {
        let mut str = String::new();
        for tok in toks {
            match &tok.tok {
                RichTokenType::Char(c) => str.push(*c),
                RichTokenType::Text(s) => str.push_str(s),
                RichTokenType::Character(_, _) => parse_error(tok.loc, ParseErrorType::CmdInCmd)?,
                RichTokenType::Command(_, _) => parse_error(tok.loc, ParseErrorType::CmdInCmd)?,
            }
        }
        Ok(str)
    }

    fn parse_program(toks: &[RichToken]) -> ParseResult<Program> {
        let program = Self::concat_params(toks)?;
        match ProgramParser::new().parse(&program) {
            Ok(p) => Ok(p),
            Err(e) => {
                use lalrpop_util::ParseError as ExecParseError;

                let loc = Loc::from_locs(toks.iter().map(|tok| tok.loc));
                let loc = match &e {
                    ExecParseError::InvalidToken { location } => {
                        Loc(loc.0 + location, loc.0 + location + 1)
                    }
                    ExecParseError::UnrecognizedEOF {
                        location: _,
                        expected: _,
                    } => Loc(loc.1, loc.1 + 1),
                    ExecParseError::UnrecognizedToken { token, expected: _ }
                    | ExecParseError::ExtraToken { token } => Loc(loc.0 + token.0, loc.0 + token.2),
                    ExecParseError::User { error: _ } => loc,
                };
                parse_error(loc, ParseErrorType::InvalidProgram(e.to_string()))
            }
        }
    }

    fn check_params_count(
        count: usize,
        min: usize,
        max: usize,
        loc: Loc,
        name: &str,
    ) -> ParseResult<()> {
        if count < min || count > max {
            parse_error(
                loc,
                ParseErrorType::InvalidParamsCount(name.to_string(), count),
            )
        } else {
            Ok(())
        }
    }

    fn parse_command(loc: Loc, name: &str, params: &[Vec<RichToken>]) -> ParseResult<Line> {
        let params_count = params.len();
        let cmd = match name {
            "res" => {
                Self::check_params_count(params_count, 1, 1, loc, name)?;
                // Construct a simple program to get the resource.
                // We don't expose this command to the front end.
                Command::Exec(Program(vec![Expr::Ref(Ref::Res(Self::concat_params(
                    &params[0],
                )?))]))
            }
            "ch" => {
                Self::check_params_count(params_count, 1, 2, loc, name)?;
                Command::Character(
                    Self::concat_params(&params[0])?,
                    Self::concat_params(
                        params.get(1).map(|slice| slice.as_slice()).unwrap_or(&[]),
                    )?,
                )
            }
            "exec" => {
                Self::check_params_count(params_count, 1, 1, loc, name)?;
                Command::Exec(Self::parse_program(&params[0])?)
            }
            "switch" => {
                Self::check_params_count(params_count, 1, 3, loc, name)?;
                let enabled = match params.get(2) {
                    Some(toks) => Some(Self::parse_program(toks)?),
                    None => None,
                };
                Command::Switch {
                    text: Self::concat_params(&params[0])?,
                    action: if let Some(toks) = params.get(1) {
                        Self::parse_program(toks)?
                    } else {
                        Program::default()
                    },
                    enabled,
                }
            }
            name => {
                let mut args = vec![];
                for p in params.iter() {
                    args.push(Self::concat_params(p)?);
                }
                Command::Other(name.to_string(), args)
            }
        };
        Ok(Line::Cmd(cmd))
    }
}

impl<'a> Iterator for TextParser<'a> {
    type Item = ParseResult<Line>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parse_next() {
            Ok(Some(res)) => Some(Ok(res)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod test_lexer {
    use crate::text::*;

    #[test]
    fn basic() {
        let lexer = TextLexer::new("\\par text");
        let res = lexer.collect::<Vec<_>>();
        assert_eq!(res.len(), 4);
        assert_eq!(res[0].tok, TokenType::SpecChar('\\'));
        assert_eq!(res[1].tok, TokenType::Text("par"));
        assert_eq!(res[2].tok, TokenType::Space);
        assert_eq!(res[3].tok, TokenType::Text("text"));
    }

    #[test]
    fn space() {
        let lexer = TextLexer::new("text \\par");
        let res = lexer.collect::<Vec<_>>();
        assert_eq!(res.len(), 4);
        assert_eq!(res[0].tok, TokenType::Text("text"));
        assert_eq!(res[1].tok, TokenType::Space);
        assert_eq!(res[2].tok, TokenType::SpecChar('\\'));
        assert_eq!(res[3].tok, TokenType::Text("par"));
    }
}

#[cfg(test)]
mod test_rich_lexer {
    use crate::text::*;

    #[test]
    fn space() {
        let mut lexer = TextRichLexer::new("\\cmd{123} \\cmd{123}");
        let res = lexer.try_collect::<Vec<_>>().unwrap();
        assert_eq!(res.len(), 3);
        assert_eq!(res[1].tok, RichTokenType::Char(' '));
    }
}

#[cfg(test)]
mod test_parser {
    use crate::{text::*, *};

    #[test]
    fn basic() {
        assert_eq!(
            TextParser::new("\\\\").parse().unwrap(),
            Text(vec![Line::Str("\\".to_string())])
        );
        assert_eq!(
            TextParser::new("\\{").parse().unwrap(),
            Text(vec![Line::Str("{".to_string())])
        );
    }

    #[test]
    fn space() {
        assert_eq!(
            TextParser::new("\\cmd{123} \\cmd{123}").parse().unwrap(),
            Text(vec![
                Line::Cmd(Command::Other("cmd".to_string(), vec!["123".to_string()])),
                Line::Str(" ".to_string()),
                Line::Cmd(Command::Other("cmd".to_string(), vec!["123".to_string()])),
            ])
        );
    }

    #[test]
    fn exec() {
        assert_eq!(
            TextParser::new(r##"\exec{"Hello world!"}"##)
                .parse()
                .unwrap(),
            Text(vec![Line::Cmd(Command::Exec(Program(vec![Expr::Const(
                RawValue::Str("Hello world!".to_string())
            )])))])
        );
        assert_eq!(
            TextParser::new(r##"\exec{"Hello world!{}"}"##)
                .parse()
                .unwrap(),
            Text(vec![Line::Cmd(Command::Exec(Program(vec![Expr::Const(
                RawValue::Str("Hello world!{}".to_string())
            )])))])
        );
        TextParser::new(r##"\exec{format.fmt("Hello {}", "world!")}"##)
            .parse()
            .unwrap();
    }

    #[test]
    fn switch() {
        assert_eq!(
            TextParser::new(r##"\switch{hello}{"Hello world!"}"##)
                .parse()
                .unwrap(),
            Text(vec![Line::Cmd(Command::Switch {
                text: "hello".to_string(),
                action: Program(vec![Expr::Const(RawValue::Str("Hello world!".to_string()))]),
                enabled: None
            })])
        );

        TextParser::new(r##"\switch{hello}{$s = 2}{a == b}"##)
            .parse()
            .unwrap();
    }

    #[test]
    fn error() {
        assert_eq!(
            TextParser::new(r##"\switch{\exec{114514}}"##).parse(),
            Err(ParseError::new(Loc(9, 13), ParseErrorType::CmdInCmd))
        );
    }

    #[test]
    fn lf() {
        assert_eq!(
            TextParser::new(" ").parse().unwrap(),
            Text(vec![Line::Str(" ".to_string())])
        );
        assert_eq!(
            TextParser::new("  ").parse().unwrap(),
            Text(vec![Line::Str(" ".to_string())])
        );
        assert_eq!(
            TextParser::new(" \n ").parse().unwrap(),
            Text(vec![Line::Str(" ".to_string())])
        );
        assert_eq!(
            TextParser::new(" 123 ").parse().unwrap(),
            Text(vec![Line::Str(" 123 ".to_string())])
        );
        assert_eq!(
            TextParser::new(" \n123\t ").parse().unwrap(),
            Text(vec![Line::Str(" 123 ".to_string())])
        );
        assert_eq!(
            TextParser::new("123").parse().unwrap(),
            Text(vec![Line::Str("123".to_string())])
        );
    }
}
