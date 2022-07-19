use gal_bindings::*;

#[export]
fn plugin_type() -> PluginType {
    PluginType::Text
}

#[export]
fn text_commands() -> Vec<&'static str> {
    vec!["bg", "bgm", "video"]
}

fn file(
    args: Vec<String>,
    ctx: TextProcessContext,
    game_prop: &str,
    prop: &str,
    exs: &[&str],
) -> TextProcessResult {
    assert_eq!(args.len(), 1);
    let bg_dir = ctx.root_path.join(&ctx.game_props[game_prop]);
    let mut res = TextProcessResult::default();
    if let Some(path) = exs
        .into_iter()
        .map(|ex| bg_dir.join(&args[0]).with_extension(ex))
        // TODO: enable fs for wasi
        //.filter(|p| p.exists())
        .next()
    {
        res.props
            .insert(prop.to_string(), path.to_string_lossy().into_owned());
    }
    res
}

#[export]
fn bg(args: Vec<String>, ctx: TextProcessContext) -> TextProcessResult {
    file(args, ctx, "bgs", "bg", &["jpg", "png", "gif"])
}

#[export]
fn bgm(args: Vec<String>, ctx: TextProcessContext) -> TextProcessResult {
    file(args, ctx, "bgms", "bgm", &["mp3"])
}

#[export]
fn video(args: Vec<String>, ctx: TextProcessContext) -> TextProcessResult {
    file(args, ctx, "videos", "video", &["mp4"])
}
