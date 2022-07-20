use gal_bindings::*;

#[export]
fn plugin_type() -> PluginType {
    PluginType::Text
}

#[export]
fn text_commands() -> Vec<&'static str> {
    vec!["par", "textrm", "textsf", "texttt"]
}

#[export]
fn par(args: Vec<String>, ctx: TextProcessContext) -> TextProcessResult {
    assert!(args.is_empty());
    let mut res = TextProcessResult::default();
    match ctx.frontend {
        FrontendType::Text => res.line.push(ActionLine::chars("\n")),
        FrontendType::Html => res.line.push(ActionLine::chars("<br />")),
    }
    res
}

fn text_font(args: Vec<String>, ctx: TextProcessContext, fonts: &str) -> TextProcessResult {
    assert_eq!(args.len(), 1);
    let mut res = TextProcessResult::default();
    match ctx.frontend {
        FrontendType::Text => res.line.push(ActionLine::chars(&args[0])),
        FrontendType::Html => {
            res.line
                .push(ActionLine::block(format!("<font face=\"{}\">", fonts)));
            res.line.push(ActionLine::chars(&args[0]));
            res.line.push(ActionLine::block("</font>"));
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
