use crate::exec::*;
use std::{error::Error, fmt::Display, str::Chars};

#[derive(Debug, PartialEq, Eq)]
pub struct Text(pub Vec<Line>);

#[derive(Debug, PartialEq, Eq)]
enum Token<'a> {
    Space,
    SpecChar(char),
    Text(&'a str),
}

#[derive(Debug, PartialEq, Eq)]
enum RichToken<'a> {
    Char(char),
    Text(&'a str),
    Command(&'a str, Vec<Vec<RichToken<'a>>>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError {
    filename: String,
    col: usize,
    row: usize,
    err: ParseErrorType,
}

impl ParseError {
    // TODO: filename & loc support.
    pub fn new(err: ParseErrorType) -> Self {
        Self {
            filename: String::new(),
            col: 0,
            row: 0,
            err,
        }
    }

    pub fn loc(&self) -> (usize, usize) {
        (self.col, self.row)
    }

    pub fn error(&self) -> &ParseErrorType {
        &self.err
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}:{}",
            self.filename, self.col, self.row, self.err
        )
    }
}

impl Error for ParseError {}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseErrorType {
    IllegalChar(char),
    CmdNotFound,
    CmdInCmd,
    InvalidCmd(String),
    InvalidParamsCount(String, usize),
    InvalidProgram(String),
}

impl Display for ParseErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IllegalChar(c) => write!(f, "Illegal char \"{}\".", c.escape_default())?,
            Self::CmdNotFound => write!(f, "Command not found after \"\\\".")?,
            Self::CmdInCmd => write!(f, "Embedded command is not supported.")?,
            Self::InvalidCmd(name) => write!(f, "Invalid commmand \"{}\"", name.escape_default())?,
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

pub type ParseResult<T> = std::result::Result<T, ParseError>;

#[derive(Debug, PartialEq, Eq)]
pub enum Line {
    Str(String),
    Cmd(Command),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Pause,
    Exec(Program),
    Switch {
        text: String,
        action: Program,
        enabled: Option<Program>,
    },
}

struct Peakable<T: Iterator> {
    iter: T,
    head: Option<T::Item>,
    readed: usize,
}

impl<T: Iterator> Peakable<T> {
    pub fn new(iter: T) -> Self {
        Self {
            iter,
            head: None,
            readed: 0,
        }
    }

    pub fn peak(&mut self) -> Option<&T::Item> {
        if self.head.is_none() {
            if let Some(item) = self.iter.next() {
                self.head = Some(item);
            } else {
                return None;
            }
        }
        self.head.as_ref()
    }

    pub fn next(&mut self) -> Option<T::Item> {
        if let Some(c) = self.head.take() {
            self.readed += 1;
            Some(c)
        } else if let Some(c) = self.iter.next() {
            self.readed += 1;
            Some(c)
        } else {
            None
        }
    }

    pub fn readed(&self) -> usize {
        self.readed
    }
}

const fn is_special_char(c: char) -> bool {
    match c {
        '\\' | '{' | '}' => true,
        _ => false,
    }
}

struct TextLexer<'a> {
    text: &'a str,
    chars: Peakable<Chars<'a>>,
}

impl<'a> TextLexer<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            chars: Peakable::new(text.chars()),
        }
    }
}

impl<'a> Iterator for TextLexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut has_whitespace = false;
        while let Some(c) = self.chars.peak() {
            if c.is_whitespace() {
                self.chars.next();
                has_whitespace = true;
            } else {
                break;
            }
        }
        if has_whitespace {
            return Some(Token::Space);
        }
        let cur = self.chars.readed();
        while let Some(&c) = self.chars.peak() {
            if is_special_char(c) {
                if self.chars.readed() - cur > 0 {
                    return Some(Token::Text(&self.text[cur..self.chars.readed()]));
                } else {
                    self.chars.next();
                    return Some(Token::SpecChar(c));
                }
            } else {
                self.chars.next();
            }
        }
        if self.chars.readed() - cur > 0 {
            Some(Token::Text(&self.text[cur..self.chars.readed()]))
        } else {
            None
        }
    }
}

struct TextRichLexer<'a> {
    lexer: Peakable<TextLexer<'a>>,
    in_param: usize,
}

impl<'a> TextRichLexer<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            lexer: Peakable::new(TextLexer::new(text)),
            in_param: 0,
        }
    }

    fn err(&self, err: ParseErrorType) -> ParseResult<!> {
        Err(ParseError::new(err))
    }

    fn parse_spec_char(&mut self, c: char) -> ParseResult<RichToken<'a>> {
        match c {
            '\\' => self.parse_escape_or_command(),
            '{' | '}' if self.in_param > 0 => Ok(RichToken::Char(c)),
            _ => self.err(ParseErrorType::IllegalChar(c))?,
        }
    }

    fn parse_escape_or_command(&mut self) -> ParseResult<RichToken<'a>> {
        if let Some(tok) = self.lexer.next() {
            match tok {
                Token::Space => Ok(RichToken::Char(' ')),
                Token::SpecChar(c) => Ok(RichToken::Char(c)),
                Token::Text(name) => {
                    if self.in_param > 0 {
                        self.err(ParseErrorType::CmdInCmd)?
                    } else {
                        self.parse_params(name)
                    }
                }
            }
        } else {
            self.err(ParseErrorType::CmdNotFound)?
        }
    }

    fn parse_params(&mut self, name: &'a str) -> ParseResult<RichToken<'a>> {
        if let Some(tok) = self.lexer.peak() {
            match tok {
                Token::Space => {
                    self.lexer.next();
                    Ok(RichToken::Command(name, vec![]))
                }
                &Token::SpecChar(c) => match c {
                    '\\' => Ok(RichToken::Command(name, vec![])),
                    '{' => {
                        let mut params = vec![];
                        while self.lexer.peak() == Some(&Token::SpecChar('{')) {
                            params.push(self.parse_param()?);
                        }
                        Ok(RichToken::Command(name, params))
                    }
                    _ => self.err(ParseErrorType::IllegalChar(c))?,
                },
                Token::Text(_) => self.err(ParseErrorType::CmdNotFound)?,
            }
        } else {
            Ok(RichToken::Command(name, vec![]))
        }
    }

    fn parse_param(&mut self) -> ParseResult<Vec<RichToken<'a>>> {
        assert_eq!(self.lexer.next(), Some(Token::SpecChar('{')));
        self.in_param += 1;
        let mut tokens = vec![];
        while let Some(tok) = self.lexer.next() {
            match tok {
                Token::Space => tokens.push(RichToken::Char(' ')),
                Token::SpecChar(c) => {
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
                    tokens.push(self.parse_spec_char(c)?);
                }
                Token::Text(s) => tokens.push(RichToken::Text(s)),
            }
        }
        Ok(tokens)
    }
}

impl<'a> Iterator for TextRichLexer<'a> {
    type Item = ParseResult<RichToken<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(tok) = self.lexer.next() {
            match tok {
                Token::Space => Some(Ok(RichToken::Char(' '))),
                Token::SpecChar(c) => Some(self.parse_spec_char(c)),
                Token::Text(s) => Some(Ok(RichToken::Text(s))),
            }
        } else {
            None
        }
    }
}

pub struct TextParser<'a> {
    lexer: Peakable<TextRichLexer<'a>>,
}

impl<'a> TextParser<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            lexer: Peakable::new(TextRichLexer::new(text)),
        }
    }

    pub fn parse(mut self) -> ParseResult<Text> {
        Ok(Text(self.try_collect()?))
    }

    fn err(&self, err: ParseErrorType) -> ParseResult<!> {
        Err(ParseError::new(err))
    }

    fn parse_next(&mut self) -> ParseResult<Option<Line>> {
        let mut str = String::new();
        while let Some(tok) = self.lexer.peak() {
            match tok {
                Ok(tok) => match tok {
                    RichToken::Char(c) => {
                        str.push(*c);
                        self.lexer.next();
                    }
                    RichToken::Text(s) => {
                        str.push_str(s);
                        self.lexer.next();
                    }
                    RichToken::Command(_, _) => {
                        if str.is_empty() {
                            let (name, params) = if let Some(Ok(RichToken::Command(name, params))) =
                                self.lexer.next()
                            {
                                (name.to_string(), params)
                            } else {
                                unreachable!()
                            };
                            let params_count = params.len();
                            let cmd = match name.as_str() {
                                "pause" => {
                                    if params_count > 0 {
                                        self.err(ParseErrorType::InvalidParamsCount(
                                            name,
                                            params_count,
                                        ))?;
                                    }
                                    Command::Pause
                                }
                                "exec" => {
                                    if params_count != 1 {
                                        self.err(ParseErrorType::InvalidParamsCount(
                                            name,
                                            params_count,
                                        ))?;
                                    }
                                    Command::Exec(self.parse_program(&params[0])?)
                                }
                                "switch" => {
                                    if params_count != 2 && params_count != 3 {
                                        self.err(ParseErrorType::InvalidParamsCount(
                                            name,
                                            params_count,
                                        ))?;
                                    }
                                    let enabled = match params.get(2) {
                                        Some(toks) => Some(self.parse_program(toks)?),
                                        None => None,
                                    };
                                    Command::Switch {
                                        text: self.concat_params(&params[0])?,
                                        action: self.parse_program(&params[1])?,
                                        enabled,
                                    }
                                }
                                _ => self.err(ParseErrorType::InvalidCmd(name))?,
                            };
                            return Ok(Some(Line::Cmd(cmd)));
                        } else {
                            break;
                        }
                    }
                },
                Err(_) => {
                    self.lexer.next().unwrap()?;
                }
            }
        }
        if !str.is_empty() {
            Ok(Some(Line::Str(str)))
        } else {
            Ok(None)
        }
    }

    fn concat_params(&self, toks: &[RichToken]) -> ParseResult<String> {
        let mut str = String::new();
        for tok in toks {
            match tok {
                RichToken::Char(c) => str.push(*c),
                RichToken::Text(s) => str.push_str(s),
                RichToken::Command(_, _) => self.err(ParseErrorType::CmdInCmd)?,
            }
        }
        Ok(str)
    }

    fn parse_program(&self, toks: &[RichToken]) -> ParseResult<Program> {
        let program = self.concat_params(toks)?;
        match ProgramParser::new().parse(&program) {
            Ok(p) => Ok(p),
            Err(e) => self.err(ParseErrorType::InvalidProgram(e.to_string()))?,
        }
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
mod test {
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
        assert_eq!(
            TextParser::new("\\pause").parse().unwrap(),
            Text(vec![Line::Cmd(Command::Pause)])
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
            Err(ParseError::new(ParseErrorType::CmdInCmd))
        );
    }
}
