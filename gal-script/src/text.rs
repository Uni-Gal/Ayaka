use crate::exec::*;
use std::{error::Error, fmt::Display, str::CharIndices};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Loc(pub usize, pub usize);

impl Loc {
    pub fn from_locs(locs: impl Iterator<Item = Loc>) -> Self {
        let mut start = usize::MAX;
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
    Par,
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
}

impl<T: Iterator> Peakable<T> {
    pub fn new(iter: T) -> Self {
        Self { iter, head: None }
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
            Some(c)
        } else if let Some(c) = self.iter.next() {
            Some(c)
        } else {
            None
        }
    }
}

struct PeakableChars<'a> {
    iter: CharIndices<'a>,
    head: Option<char>,
    readed: usize,
}

impl<'a> PeakableChars<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            iter: s.char_indices(),
            head: None,
            readed: 0,
        }
    }

    pub fn peak(&mut self) -> Option<char> {
        if self.head.is_none() {
            if let Some((_, item)) = self.iter.next() {
                self.head = Some(item);
            } else {
                return None;
            }
        }
        self.head.clone()
    }

    pub fn next(&mut self) -> Option<char> {
        if let Some(c) = self.head.take() {
            self.readed = self.iter.offset();
            Some(c)
        } else if let Some((index, c)) = self.iter.next() {
            self.readed = index;
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
    chars: PeakableChars<'a>,
}

impl<'a> TextLexer<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            chars: PeakableChars::new(text),
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
        while let Some(c) = self.chars.peak() {
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
                        while let Some(tok) = self.lexer.peak() {
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
                            return Ok(Some(self.parse_command(loc, name, params)?));
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
                self.err(loc, ParseErrorType::InvalidProgram(e.to_string()))?
            }
        }
    }

    fn check_params_count(
        &self,
        count: usize,
        min: usize,
        max: usize,
        loc: Loc,
        name: String,
    ) -> ParseResult<()> {
        if count < min || count > max {
            self.err(loc, ParseErrorType::InvalidParamsCount(name, count))?;
        }
        Ok(())
    }

    fn parse_command(
        &self,
        loc: Loc,
        name: String,
        params: Vec<Vec<RichToken>>,
    ) -> ParseResult<Line> {
        let params_count = params.len();
        let cmd = match name.as_str() {
            "pause" => {
                self.check_params_count(params_count, 0, 0, loc, name)?;
                Command::Pause
            }
            "par" => {
                self.check_params_count(params_count, 0, 0, loc, name)?;
                Command::Par
            }
            "exec" => {
                self.check_params_count(params_count, 1, 1, loc, name)?;
                Command::Exec(self.parse_program(&params[0])?)
            }
            "switch" => {
                self.check_params_count(params_count, 2, 3, loc, name)?;
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
