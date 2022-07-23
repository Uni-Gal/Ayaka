use gal_bindings::*;

#[export]
fn plugin_type() -> PluginType {
    PluginType::Text
}

#[export]
fn text_commands() -> &'static [&'static str] {
    &["par", "textrm", "textsf", "texttt", "ruby"]
}

#[export]
fn par(args: Vec<String>, ctx: TextProcessContext) -> TextProcessResult {
    assert!(args.is_empty());
    let mut res = TextProcessResult::default();
    match ctx.frontend {
        FrontendType::Text => res.line.push_back_chars("\n"),
        FrontendType::Html => res.line.push_back_chars("<br />"),
    }
    res
}

fn text_font(args: Vec<String>, ctx: TextProcessContext, fonts: &str) -> TextProcessResult {
    assert_eq!(args.len(), 1);
    let mut res = TextProcessResult::default();
    match ctx.frontend {
        FrontendType::Text => res.line.push_back_chars(&args[0]),
        FrontendType::Html => {
            res.line
                .push_back_block(format!("<font face=\"{}\">", fonts));
            res.line.push_back_chars(&args[0]);
            res.line.push_back_block("</font>");
        }
    }
    res
}

#[export]
fn textrm(args: Vec<String>, ctx: TextProcessContext) -> TextProcessResult {
    text_font(args, ctx, "Times New Roman")
}

#[export]
fn textsf(args: Vec<String>, ctx: TextProcessContext) -> TextProcessResult {
    text_font(args, ctx, "Arial")
}

#[export]
fn texttt(args: Vec<String>, ctx: TextProcessContext) -> TextProcessResult {
    text_font(args, ctx, "Courier New")
}

#[export]
fn ruby(args: Vec<String>, ctx: TextProcessContext) -> TextProcessResult {
    assert_eq!(args.len(), 2);
    let mut res = TextProcessResult::default();
    match ctx.frontend {
        FrontendType::Text => res
            .line
            .push_back_chars(format!("{}（{}）", args[0], args[1])),
        FrontendType::Html => {
            res.line.push_back_block("<ruby>");
            res.line.push_back_chars(&args[0]);
            res.line.push_back_block("<rp>（</rp><rt>");
            res.line.push_back_chars(&args[1]);
            res.line.push_back_block("</rt><rp>）</rp>");
            res.line.push_back_block("</ruby>");
        }
    }
    res
}
