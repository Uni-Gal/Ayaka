#![deny(unsafe_code)]

use ayaka_bindings::*;

#[export]
fn plugin_type() -> PluginType {
    PluginType::builder()
        .text(["par", "textrm", "textsf", "texttt", "ruby"])
        .build()
}

#[export]
fn par(args: Vec<String>, ctx: TextProcessContext) -> TextProcessResult {
    assert!(args.is_empty());
    let mut res = TextProcessResult::default();
    match ctx.frontend {
        FrontendType::Text => res.text.push_back_chars("\n"),
        FrontendType::Html => res.text.push_back_block("<br />"),
        FrontendType::Latex => res.text.push_back_block("\\par "),
    }
    res
}

fn text_font(
    cmd: &str,
    args: Vec<String>,
    ctx: TextProcessContext,
    fonts: &str,
) -> TextProcessResult {
    assert_eq!(args.len(), 1);
    let mut res = TextProcessResult::default();
    match ctx.frontend {
        FrontendType::Text => res.text.push_back_chars(&args[0]),
        FrontendType::Html => {
            res.text
                .push_back_block(format!("<font face=\"{}\">", fonts));
            res.text.push_back_chars(&args[0]);
            res.text.push_back_block("</font>");
        }
        FrontendType::Latex => {
            res.text.push_back_block(format!("\\{}{{", cmd));
            res.text.push_back_chars(&args[0]);
            res.text.push_back_block("}");
        }
    }
    res
}

#[export]
fn textrm(args: Vec<String>, ctx: TextProcessContext) -> TextProcessResult {
    text_font("textrm", args, ctx, "Times New Roman")
}

#[export]
fn textsf(args: Vec<String>, ctx: TextProcessContext) -> TextProcessResult {
    text_font("textsf", args, ctx, "Arial")
}

#[export]
fn texttt(args: Vec<String>, ctx: TextProcessContext) -> TextProcessResult {
    text_font("texttt", args, ctx, "Courier New")
}

#[export]
fn ruby(args: Vec<String>, ctx: TextProcessContext) -> TextProcessResult {
    assert_eq!(args.len(), 2);
    let mut res = TextProcessResult::default();
    match ctx.frontend {
        FrontendType::Text => res
            .text
            .push_back_chars(format!("{}（{}）", args[0], args[1])),
        FrontendType::Html => {
            res.text.push_back_block("<ruby>");
            res.text.push_back_chars(&args[0]);
            res.text.push_back_block("<rp>（</rp><rt>");
            res.text.push_back_chars(&args[1]);
            res.text.push_back_block("</rt><rp>）</rp>");
            res.text.push_back_block("</ruby>");
        }
        FrontendType::Latex => res
            .text
            .push_back_block(format!("\\ruby{{{}}}{{{}}}", args[0], args[1])),
    }
    res
}
