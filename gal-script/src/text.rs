use crate::exec::*;
use std::{error::Error, fmt::Display, str::Chars};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Loc(usize, usize);

impl Loc {
    pub fn from_locs(locs: impl Iterator<Item = Loc>) -> Self {
        let mut start = 0;
        let mut end = 0;
        for loc in locs {
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

    pub fn command(loc: Loc, n: &'a str, params: Vec<Vec<RichToken<'a>>>) -> Self {
        Self {
            loc,
            tok: RichTokenType::Command(n, params),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError {
    loc: Loc,
    err: ParseErrorType,
}

impl ParseError {
    pub fn new(loc: Loc, err: ParseErrorType) -> Self {
        Self { loc, err }
    }

    pub fn loc(&self) -> Loc {
        self.loc
    }

    pub fn error(&self) -> &ParseErrorType {
        &self.err
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.err.fmt(f)
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

#[derive(Debug, PartialEq, Eq)]
pub struct Text(pub Vec<Line>);

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

    pub fn err(&self, loc: Loc, err: ParseErrorType) -> ParseResult<!> {
        Err(ParseError::new(loc, err))
    }
}

impl<'a> Iterator for TextLexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.chars.readed();
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
            return Some(Token::space(Loc(cur, self.chars.readed())));
        }
        let cur = self.chars.readed();
        while let Some(&c) = self.chars.peak() {
            if is_special_char(c) {
                if self.chars.readed() - cur > 0 {
                    return Some(Token::text(
                        Loc(cur, self.chars.readed()),
                        &self.text[cur..self.chars.readed()],
                    ));
                } else {
                    self.chars.next();
                    return Some(Token::spec_char(
                        Loc(self.chars.readed() - 1, self.chars.readed()),
                        c,
                    ));
                }
            } else {
                self.chars.next();
            }
        }
        if self.chars.readed() - cur > 0 {
            Some(Token::text(
                Loc(cur, self.chars.readed()),
                &self.text[cur..self.chars.readed()],
            ))
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

    fn err(&self, loc: Loc, err: ParseErrorType) -> ParseResult<!> {
        self.lexer.iter.err(loc, err)
    }

    fn parse_spec_char(&mut self, loc: Loc, c: char) -> ParseResult<RichToken<'a>> {
        match c {
            '\\' => self.parse_escape_or_command(loc),
            '{' | '}' if self.in_param > 0 => Ok(RichToken::char(loc, c)),
            _ => self.err(loc, ParseErrorType::IllegalChar(c))?,
        }
    }

    fn parse_escape_or_command(&mut self, prev_loc: Loc) -> ParseResult<RichToken<'a>> {
        if let Some(tok) = self.lexer.next() {
            match tok.tok {
                TokenType::Space => Ok(RichToken::char(tok.loc, ' ')),
                TokenType::SpecChar(c) => Ok(RichToken::char(tok.loc, c)),
                TokenType::Text(name) => {
                    if self.in_param > 0 {
                        self.err(tok.loc, ParseErrorType::CmdInCmd)?
                    } else {
                        self.parse_params(Loc::from_locs([prev_loc, tok.loc].into_iter()), name)
                    }
                }
            }
        } else {
            self.err(prev_loc, ParseErrorType::CmdNotFound)?
        }
    }

    fn parse_params(&mut self, prev_loc: Loc, name: &'a str) -> ParseResult<RichToken<'a>> {
        if let Some(tok) = self.lexer.peak() {
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
                        let mut locs = vec![prev_loc];
                        while let Some(tok) = self.lexer.peak() {
                            if tok.tok == TokenType::SpecChar('{') {
                                self.lexer.next();
                                let param = self.parse_param()?;
                                locs.push(Loc::from_locs(param.iter().map(|tok| tok.loc)));
                                params.push(param);
                            } else {
                                break;
                            }
                        }
                        Ok(RichToken::command(
                            Loc::from_locs(locs.into_iter()),
                            name,
                            params,
                        ))
                    }
                    _ => self.err(loc, ParseErrorType::IllegalChar(c))?,
                },
                TokenType::Text(_) => self.err(loc, ParseErrorType::CmdNotFound)?,
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

    fn err(&self, loc: Loc, err: ParseErrorType) -> ParseResult<!> {
        self.lexer.iter.err(loc, err)
    }

    fn parse_next(&mut self) -> ParseResult<Option<Line>> {
        let mut str = String::new();
        while let Some(tok) = self.lexer.peak() {
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
                    RichTokenType::Command(_, _) => {
                        if str.is_empty() {
                            let (loc, name, params) = if let Some(Ok(RichToken {
                                loc,
                                tok: RichTokenType::Command(name, params),
                            })) = self.lexer.next()
                            {
                                (loc, name.to_string(), params)
                            } else {
                                unreachable!()
                            };
                            let params_count = params.len();
                            let cmd = match name.as_str() {
                                "pause" => {
                                    if params_count > 0 {
                                        self.err(
                                            loc,
                                            ParseErrorType::InvalidParamsCount(name, params_count),
                                        )?;
                                    }
                                    Command::Pause
                                }
                                "exec" => {
                                    if params_count != 1 {
                                        self.err(
                                            loc,
                                            ParseErrorType::InvalidParamsCount(name, params_count),
                                        )?;
                                    }
                                    Command::Exec(self.parse_program(&params[0])?)
                                }
                                "switch" => {
                                    if params_count != 2 && params_count != 3 {
                                        self.err(
                                            loc,
                                            ParseErrorType::InvalidParamsCount(name, params_count),
                                        )?;
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
                                _ => self.err(loc, ParseErrorType::InvalidCmd(name))?,
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
            Ok(Some(Line::Str(str.trim().to_string())))
        } else {
            Ok(None)
        }
    }

    fn concat_params(&self, toks: &[RichToken]) -> ParseResult<String> {
        let mut str = String::new();
        for tok in toks {
            match &tok.tok {
                RichTokenType::Char(c) => str.push(*c),
                RichTokenType::Text(s) => str.push_str(s),
                RichTokenType::Command(_, _) => self.err(tok.loc, ParseErrorType::CmdInCmd)?,
            }
        }
        Ok(str)
    }

    fn parse_program(&self, toks: &[RichToken]) -> ParseResult<Program> {
        let program = self.concat_params(toks)?;
        match ProgramParser::new().parse(&program) {
            Ok(p) => Ok(p),
            Err(e) => self.err(
                Loc::from_locs(toks.iter().map(|tok| tok.loc)),
                ParseErrorType::InvalidProgram(e.to_string()),
            )?,
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
            Err(ParseError::new(Loc(9, 13), ParseErrorType::CmdInCmd))
        );
    }

    #[test]
    fn lf() {
        assert_eq!(
            TextParser::new(" \n ").parse().unwrap(),
            Text(vec![Line::Str(String::default())])
        );
    }
}
