use gal_bindings::*;

#[export]
fn plugin_type() -> PluginType {
    PluginType::ACTION | PluginType::TEXT
}

#[export]
fn text_commands() -> &'static [&'static str] {
    &["bg", "bgm", "video"]
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
        .filter(|p| p.exists())
        .next()
    {
        res.props
            .insert(prop.to_string(), path.to_string_lossy().into_owned());
    }
    res
}

#[export]
fn bg(args: Vec<String>, ctx: TextProcessContext) -> TextProcessResult {
    file(args, ctx, "bgs", "bg", &["png", "jpg", "gif"])
}

#[export]
fn bgm(args: Vec<String>, ctx: TextProcessContext) -> TextProcessResult {
    file(args, ctx, "bgms", "bgm", &["mp3"])
}

#[export]
fn video(args: Vec<String>, ctx: TextProcessContext) -> TextProcessResult {
    file(args, ctx, "videos", "video", &["mp4"])
}

#[export]
fn process_action(mut ctx: ActionProcessContext) -> Action {
    if let Some(last_action) = ctx.last_action {
        for prop in ["bg", "bgm"] {
            if let Some(value) = last_action.props.get(prop) {
                if !ctx.action.props.contains_key(prop) {
                    ctx.action.props.insert(prop.to_string(), value.clone());
                }
            }
        }
    }
    ctx.action
}
