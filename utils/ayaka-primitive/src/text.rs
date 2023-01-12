//! The text parser.

use crate::*;
use nom::{
    branch::alt,
    bytes::complete::{take_till, take_till1, take_until, take_while, take_while1},
    character::complete::{char, one_of},
    combinator::{all_consuming, iterator, map},
    error::VerboseError,
    multi::many0,
    sequence::{delimited, terminated},
    *,
};
use serde::Deserialize;
use trylog::macros::*;

/// A collection of [`SubText`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(from = "RawValue")]
pub struct Text {
    /// The tag of current character.
    pub ch_tag: Option<String>,
    /// The alias of current character.
    pub ch_alias: Option<String>,
    /// The texts.
    pub sub_texts: Vec<SubText>,
}

/// A part of a line, either some texts or a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubText {
    /// Special character
    Char(char),
    /// Raw texts.
    Str(String),
    /// A TeX-like command in the text.
    Cmd(String, Vec<SubText>),
}

type Res<I, O> = IResult<I, O, VerboseError<I>>;

fn take_space(i: &str) -> Res<&str, &str> {
    take_while(|c: char| c.is_whitespace())(i)
}

fn take_cmd(i: &str) -> Res<&str, &str> {
    take_while1(|c: char| c.is_ascii_alphabetic())(i)
}

fn is_str_end(c: char) -> bool {
    c.is_whitespace() || c == '\\' || c == '{' || c == '}'
}

fn parse_arg(i: &str) -> Res<&str, SubText> {
    let (i, _) = take_space(i)?;
    let (i, sub_text) = delimited(char('{'), parse_sub_text, char('}'))(i)?;
    Ok((i, sub_text))
}

fn parse_escape_command(i: &str) -> Res<&str, SubText> {
    let (i, cmd) = take_cmd(i)?;
    let (i, args) = many0(parse_arg)(i)?;
    Ok((i, SubText::Cmd(cmd.to_string(), args)))
}

fn parse_escape_char(i: &str) -> Res<&str, SubText> {
    let (i, c) = one_of("\\{}/")(i)?;
    Ok((i, SubText::Char(c)))
}

fn parse_sub_text_escape(i: &str) -> Res<&str, SubText> {
    let (i, _) = char('\\')(i)?;
    alt((parse_escape_char, parse_escape_command))(i)
}

fn parse_sub_text_str(i: &str) -> Res<&str, SubText> {
    let (i, pre_space) = take_space(i)?;
    let (i, str) = if pre_space.is_empty() {
        take_till1(is_str_end)(i)
    } else {
        take_till(is_str_end)(i)
    }?;
    let (i, post_space) = take_space(i)?;
    let str = format!(
        "{}{}{}",
        if !pre_space.is_empty() { " " } else { "" },
        str,
        if !post_space.is_empty() { " " } else { "" }
    );
    Ok((i, SubText::Str(str)))
}

fn parse_sub_text(i: &str) -> Res<&str, SubText> {
    alt((parse_sub_text_escape, parse_sub_text_str))(i)
}

fn parse_sub_texts(i: &str) -> Res<&str, Vec<SubText>> {
    let mut it = iterator(i, parse_sub_text);
    let sub_texts = it.collect();
    let (i, ()) = it.finish()?;
    Ok((i, sub_texts))
}

fn parse_text_without_ch(i: &str) -> Res<&str, Text> {
    let (i, sub_texts) = parse_sub_texts(i)?;
    let text = Text {
        ch_tag: None,
        ch_alias: None,
        sub_texts,
    };
    Ok((i, text))
}

fn parse_text_with_ch(i: &str) -> Res<&str, Text> {
    let (i, _) = char('/')(i)?;
    let (i, ch_tag) = map(terminated(take_until("/"), char('/')), str::trim)(i)?;
    let (i, ch_alias) = map(terminated(take_until("/"), char('/')), str::trim)(i)?;
    let (i, mut text) = parse_text_without_ch(i)?;
    if !ch_tag.is_empty() {
        text.ch_tag = Some(ch_tag.to_string());
    }
    if !ch_alias.is_empty() {
        text.ch_alias = Some(ch_alias.to_string());
    }
    Ok((i, text))
}

fn parse_text(i: &str) -> Res<&str, Text> {
    all_consuming(alt((parse_text_with_ch, parse_text_without_ch)))(i)
}

impl From<RawValue> for Text {
    fn from(value: RawValue) -> Self {
        let (_, text) = unwrap_or_default_log!(
            parse_text(&value.get_str()),
            "Text parse error, fallback to default"
        );
        text
    }
}

#[cfg(test)]
pub mod test {
    use crate::text::{parse_text, SubText, Text};

    pub fn text(sub_texts: Vec<SubText>) -> Text {
        Text {
            ch_tag: None,
            ch_alias: None,
            sub_texts,
        }
    }

    pub fn text_ch(tag: Option<&str>, alias: Option<&str>, sub_texts: Vec<SubText>) -> Text {
        Text {
            ch_tag: tag.map(|s| s.into()),
            ch_alias: alias.map(|s| s.into()),
            sub_texts,
        }
    }

    pub fn char(c: char) -> SubText {
        SubText::Char(c)
    }

    pub fn str(s: impl Into<String>) -> SubText {
        SubText::Str(s.into())
    }

    pub fn cmd(cmd: impl Into<String>, args: Vec<SubText>) -> SubText {
        SubText::Cmd(cmd.into(), args)
    }

    #[test]
    fn basic() {
        assert_eq!(parse_text("\\\\").unwrap().1, text(vec![char('\\')]));
        assert_eq!(parse_text("\\{").unwrap().1, text(vec![char('{')]));
    }

    #[test]
    fn space() {
        assert_eq!(
            parse_text("\\cmd{123} \\cmd{123}").unwrap().1,
            text(vec![
                cmd("cmd", vec![str("123")]),
                str(" "),
                cmd("cmd", vec![str("123")]),
            ])
        );
        assert_eq!(
            parse_text("\\par \\cmd{123}").unwrap().1,
            text(vec![
                cmd("par", vec![]),
                str(" "),
                cmd("cmd", vec![str("123")])
            ])
        );
    }

    #[test]
    fn embedded() {
        assert_eq!(
            parse_text(r##"\switch{\exec{114514}}"##).unwrap().1,
            text(vec![cmd("switch", vec![cmd("exec", vec![str("114514")])])])
        );
    }

    #[test]
    fn lf() {
        assert_eq!(parse_text(" ").unwrap().1, text(vec![str(" ")]));
        assert_eq!(parse_text("  ").unwrap().1, text(vec![str(" ")]));
        assert_eq!(parse_text(" \n ").unwrap().1, text(vec![str(" ")]));
        assert_eq!(parse_text(" 123 ").unwrap().1, text(vec![str(" 123 ")]));
        assert_eq!(parse_text(" \n123\t ").unwrap().1, text(vec![str(" 123 ")]));
        assert_eq!(parse_text("123").unwrap().1, text(vec![str("123")]));
    }

    #[test]
    fn character() {
        assert_eq!(
            parse_text("/ch//").unwrap().1,
            text_ch(Some("ch"), None, vec![])
        );
        assert_eq!(
            parse_text("/ch/alias/").unwrap().1,
            text_ch(Some("ch"), Some("alias"), vec![])
        );
        assert_eq!(parse_text("/ / /").unwrap().1, text_ch(None, None, vec![]));
    }
}
