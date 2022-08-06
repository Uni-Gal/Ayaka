use std::path::Path;

use gal_bindings::*;

#[export]
fn plugin_type() -> PluginType {
    PluginType::ACTION | PluginType::TEXT
}

#[export]
fn text_commands() -> &'static [&'static str] {
    &["bg", "bgm", "efm", "video"]
}

fn file(
    args: Vec<String>,
    base_dir: impl AsRef<Path>,
    prop: &str,
    exs: &[&str],
) -> TextProcessResult {
    assert_eq!(args.len(), 1);
    let base_dir = base_dir.as_ref();
    log::info!(
        "File {:?}, {:?}, {}, {:?}",
        args,
        base_dir.display(),
        prop,
        exs
    );
    let mut res = TextProcessResult::default();
    if let Some(path) = exs
        .into_iter()
        .map(|ex| base_dir.join(&args[0]).with_extension(ex))
        .filter(|p| p.exists())
        .next()
    {
        res.props
            .insert(prop.to_string(), path.to_string_lossy().into_owned());
    }
    res
}

fn file_ctx(
    args: Vec<String>,
    ctx: TextProcessContext,
    game_prop: &str,
    prop: &str,
    exs: &[&str],
) -> TextProcessResult {
    file(
        args,
        ctx.root_path.join(
            ctx.game_props
                .get(game_prop)
                .map(|s| s.as_str())
                .unwrap_or(""),
        ),
        prop,
        exs,
    )
}

#[export]
fn bg(args: Vec<String>, ctx: TextProcessContext) -> TextProcessResult {
    file_ctx(args, ctx, "bgs", "bg", &["png", "jpg", "gif"])
}

#[export]
fn bgm(args: Vec<String>, ctx: TextProcessContext) -> TextProcessResult {
    file_ctx(args, ctx, "bgms", "bgm", &["mp3"])
}

#[export]
fn efm(args: Vec<String>, ctx: TextProcessContext) -> TextProcessResult {
    file_ctx(args, ctx, "efms", "efm", &["mp3"])
}

#[export]
fn video(args: Vec<String>, ctx: TextProcessContext) -> TextProcessResult {
    file_ctx(args, ctx, "videos", "video", &["mp4"])
}

#[export]
fn process_action(mut ctx: ActionProcessContext) -> Action {
    if let Some(last_action) = &ctx.last_action {
        for prop in ["bg", "bgm"] {
            if let Some(value) = last_action.props.get(prop) {
                ctx.action
                    .props
                    .entry(prop.to_string())
                    .or_insert(value.clone());
            }
        }
    }
    let voice_id = ctx.action.cur_act.to_string();
    let res = file(
        vec![voice_id],
        ctx.root_path
            .join(
                ctx.game_props
                    .get("voices")
                    .map(|s| s.as_str())
                    .unwrap_or(""),
            )
            .join(&ctx.action.cur_para),
        "voice",
        &["mp3"],
    );
    for (key, value) in res.props.into_iter() {
        ctx.action.props.insert(key, value);
    }
    ctx.action
}
