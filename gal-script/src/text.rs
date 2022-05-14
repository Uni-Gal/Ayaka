use crate::exec::*;
use std::str::Chars;

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

    fn parse_spec_char(&mut self, c: char) -> RichToken<'a> {
        match c {
            '\\' => self.parse_escape_or_command(),
            '{' | '}' if self.in_param > 0 => RichToken::Char(c),
            _ => panic!("Illegal char \"{}\"", c),
        }
    }

    fn parse_escape_or_command(&mut self) -> RichToken<'a> {
        if let Some(tok) = self.lexer.next() {
            match tok {
                Token::Space => RichToken::Char(' '),
                Token::SpecChar(c) => RichToken::Char(c),
                Token::Text(name) => {
                    if self.in_param > 0 {
                        unimplemented!()
                    } else {
                        self.parse_params(name)
                    }
                }
            }
        } else {
            panic!("Single \"\\\"")
        }
    }

    fn parse_params(&mut self, name: &'a str) -> RichToken<'a> {
        if let Some(tok) = self.lexer.peak() {
            match tok {
                Token::Space => {
                    self.lexer.next();
                    RichToken::Command(name, vec![])
                }
                Token::SpecChar(c) => match c {
                    '\\' => RichToken::Command(name, vec![]),
                    '{' => {
                        let mut params = vec![];
                        while self.lexer.peak() == Some(&Token::SpecChar('{')) {
                            params.push(self.parse_param());
                        }
                        RichToken::Command(name, params)
                    }
                    _ => panic!("Illegal char \"{}\"", c),
                },
                Token::Text(_) => unreachable!("Cannot put text directly after command"),
            }
        } else {
            RichToken::Command(name, vec![])
        }
    }

    fn parse_param(&mut self) -> Vec<RichToken<'a>> {
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
                    tokens.push(self.parse_spec_char(c));
                }
                Token::Text(s) => tokens.push(RichToken::Text(s)),
            }
        }
        tokens
    }
}

impl<'a> Iterator for TextRichLexer<'a> {
    type Item = RichToken<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(tok) = self.lexer.next() {
            match tok {
                Token::Space => Some(RichToken::Char(' ')),
                Token::SpecChar(c) => Some(self.parse_spec_char(c)),
                Token::Text(s) => Some(RichToken::Text(s)),
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

    pub fn parse(self) -> Text {
        Text(self.collect())
    }
}

fn concat_params(toks: &[RichToken]) -> String {
    let mut str = String::new();
    for tok in toks {
        match tok {
            RichToken::Char(c) => str.push(*c),
            RichToken::Text(s) => str.push_str(s),
            RichToken::Command(_, _) => {
                panic!("Cannot embed command in another command.")
            }
        }
    }
    str
}

fn parse_program(toks: &[RichToken]) -> Program {
    ProgramParser::new().parse(&concat_params(toks)).unwrap()
}

impl<'a> Iterator for TextParser<'a> {
    type Item = Line;

    fn next(&mut self) -> Option<Self::Item> {
        let mut str = String::new();
        while let Some(tok) = self.lexer.peak() {
            match tok {
                RichToken::Char(c) => {
                    str.push(*c);
                    self.lexer.next();
                }
                RichToken::Text(s) => {
                    str.push_str(s);
                    self.lexer.next();
                }
                RichToken::Command(name, params) => {
                    if str.is_empty() {
                        let cmd = match *name {
                            "pause" => {
                                assert!(params.is_empty());
                                Command::Pause
                            }
                            "exec" => {
                                assert_eq!(params.len(), 1);
                                Command::Exec(parse_program(&params[0]))
                            }
                            "switch" => {
                                assert!(params.len() == 2 || params.len() == 3);
                                Command::Switch {
                                    text: concat_params(&params[0]),
                                    action: parse_program(&params[1]),
                                    enabled: params.get(2).map(|toks| parse_program(toks)),
                                }
                            }
                            _ => panic!("Invalid command \"{}\"", name),
                        };
                        self.lexer.next();
                        return Some(Line::Cmd(cmd));
                    } else {
                        return Some(Line::Str(str));
                    }
                }
            }
        }
        if !str.is_empty() {
            Some(Line::Str(str))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{text::*, *};

    #[test]
    fn basic() {
        assert_eq!(
            TextParser::new("\\\\").parse(),
            Text(vec![Line::Str("\\".to_string())])
        );
        assert_eq!(
            TextParser::new("\\{").parse(),
            Text(vec![Line::Str("{".to_string())])
        );
        assert_eq!(
            TextParser::new("\\pause").parse(),
            Text(vec![Line::Cmd(Command::Pause)])
        );
    }

    #[test]
    fn exec() {
        assert_eq!(
            TextParser::new(r##"\exec{"Hello world!"}"##).parse(),
            Text(vec![Line::Cmd(Command::Exec(Program(vec![Expr::Const(
                RawValue::Str("Hello world!".to_string())
            )])))])
        );
        assert_eq!(
            TextParser::new(r##"\exec{"Hello world!{}"}"##).parse(),
            Text(vec![Line::Cmd(Command::Exec(Program(vec![Expr::Const(
                RawValue::Str("Hello world!{}".to_string())
            )])))])
        );
        TextParser::new(r##"\exec{format.fmt("Hello {}", "world!")}"##).parse();
    }

    #[test]
    fn switch() {
        assert_eq!(
            TextParser::new(r##"\switch{hello}{"Hello world!"}"##).parse(),
            Text(vec![Line::Cmd(Command::Switch {
                text: "hello".to_string(),
                action: Program(vec![Expr::Const(RawValue::Str("Hello world!".to_string()))]),
                enabled: None
            })])
        );

        TextParser::new(r##"\switch{hello}{$s = 2}{a == b}"##).parse();
    }
}
