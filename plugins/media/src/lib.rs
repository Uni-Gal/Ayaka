use std::path::{Path, PathBuf};

use ayaka_bindings::*;

#[export]
fn plugin_type() -> PluginType {
    PluginType::builder()
        .action()
        .text(["bg", "bgm", "efm", "video"])
        .game()
        .build()
}

fn find_exists(name: &str, base_dir: Option<&Path>, exs: &[&str]) -> Option<PathBuf> {
    base_dir.and_then(|base_dir| {
        exs.iter()
            .map(|ex| base_dir.join(name).with_extension(ex))
            .find(|p| p.exists())
    })
}

fn file(args: Vec<String>, base_dir: Option<&Path>, prop: &str, exs: &[&str]) -> TextProcessResult {
    assert_eq!(args.len(), 1);
    log::debug!(
        "File {:?}, {:?}, {}, {:?}",
        args,
        base_dir.map(|p| p.display()),
        prop,
        exs
    );
    let mut res = TextProcessResult::default();
    if let Some(path) = find_exists(&args[0], base_dir, exs) {
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
        ctx.game_props
            .get(game_prop)
            .map(|game_prop| ctx.root_path.join(game_prop))
            .as_deref(),
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
                    .or_insert_with(|| value.clone());
            }
        }
    }
    let voice_id = ctx.action.ctx.cur_act.to_string();
    let res = file(
        vec![voice_id],
        ctx.game_props
            .get("voices")
            .map(|p| ctx.root_path.join(p).join(&ctx.action.ctx.cur_para))
            .as_deref(),
        "voice",
        &["mp3"],
    );
    for (key, value) in res.props.into_iter() {
        ctx.action.props.insert(key, value);
    }
    ctx.action
}

#[export]
fn process_game(mut ctx: GameProcessContext) -> GameProcessResult {
    let base_dir = ctx.props.get("bgs").map(|p| ctx.root_path.join(p));
    if let Some(bg) = ctx.props.get_mut("bg") {
        if let Some(path) = find_exists(bg, base_dir.as_deref(), &["png", "jpg", "gif"]) {
            *bg = path.to_string_lossy().into_owned();
        }
    }
    GameProcessResult { props: ctx.props }
}
