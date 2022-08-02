use gal_bindings::*;

#[export]
fn plugin_type() -> PluginType {
    PluginType::ACTION | PluginType::TEXT
}

#[export]
fn text_commands() -> &'static [&'static str] {
    &["bg", "bgm", "efm", "voice", "video"]
}

fn file(
    args: Vec<String>,
    ctx: TextProcessContext,
    game_prop: &str,
    prop: &str,
    exs: &[&str],
) -> TextProcessResult {
    assert_eq!(args.len(), 1);
    log::info!(
        "File {:?}, {:?}, {}, {}, {:?}",
        args,
        ctx,
        game_prop,
        prop,
        exs
    );
    let res_dir = ctx.root_path.join(
        ctx.game_props
            .get(game_prop)
            .map(|s| s.as_str())
            .unwrap_or(""),
    );
    let mut res = TextProcessResult::default();
    if let Some(path) = exs
        .into_iter()
        .map(|ex| res_dir.join(&args[0]).with_extension(ex))
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
fn efm(args: Vec<String>, ctx: TextProcessContext) -> TextProcessResult {
    file(args, ctx, "efms", "efm", &["mp3"])
}

#[export]
fn voice(args: Vec<String>, _ctx: TextProcessContext) -> TextProcessResult {
    assert_eq!(args.len(), 1);
    let mut res = TextProcessResult::default();
    res.props.insert("voice_id".to_string(), args[0].clone());
    res
}

#[export]
fn video(args: Vec<String>, ctx: TextProcessContext) -> TextProcessResult {
    file(args, ctx, "videos", "video", &["mp4"])
}

#[export]
fn process_action(mut ctx: ActionProcessContext) -> Action {
    if let Some(last_action) = &ctx.last_action {
        for prop in ["bg", "bgm"] {
            if let Some(value) = last_action.props.get(prop) {
                if !ctx.action.props.contains_key(prop) {
                    ctx.action.props.insert(prop.to_string(), value.clone());
                }
            }
        }
    }
    let voice_id = ctx
        .action
        .props
        .get("voice_id")
        .map(|s| s.parse::<usize>().unwrap())
        .or_else(|| {
            ctx.last_action
                .as_ref()
                .and_then(|act| act.props.get("voice_id"))
                .map(|s| s.parse::<usize>().unwrap() + 1)
        })
        .unwrap_or_default()
        .to_string();
    ctx.action
        .props
        .insert("voice_id".to_string(), voice_id.clone());
    let res = file(
        vec![voice_id],
        TextProcessContext {
            root_path: ctx.root_path,
            game_props: ctx.game_props,
            frontend: ctx.frontend,
        },
        "voices",
        "voice",
        &["mp3"],
    );
    for (key, value) in res.props.into_iter() {
        ctx.action.props.insert(key, value);
    }
    ctx.action
}
