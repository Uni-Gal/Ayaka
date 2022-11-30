use std::path::{Path, PathBuf};

use ayaka_bindings::*;

#[export]
fn plugin_type() -> PluginType {
    PluginType::builder()
        .action()
        .line(["bg", "bgm", "video"])
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

fn file(
    arg: &str,
    base_dir: Option<&Path>,
    prop: &str,
    exs: &[&str],
    temp: bool,
) -> LineProcessResult {
    log::debug!(
        "File {}, {:?}, {}, {:?}",
        arg,
        base_dir.map(|p| p.display()),
        prop,
        exs
    );
    let mut res = LineProcessResult::default();
    if let Some(path) = find_exists(arg, base_dir, exs) {
        if temp { &mut res.vars } else { &mut res.locals }.insert(
            prop.to_string(),
            RawValue::Str(path.to_string_lossy().into_owned()),
        );
    }
    res
}

fn file_ctx(
    ctx: LineProcessContext,
    game_prop: &str,
    prop: &str,
    exs: &[&str],
    temp: bool,
) -> LineProcessResult {
    file(
        &ctx.props[prop].get_str(),
        ctx.game_props.get(game_prop).map(AsRef::<Path>::as_ref),
        prop,
        exs,
        temp,
    )
}

#[export]
fn bg(ctx: LineProcessContext) -> LineProcessResult {
    file_ctx(ctx, "bgs", "bg", &["png", "jpg", "gif"], false)
}

#[export]
fn bgm(ctx: LineProcessContext) -> LineProcessResult {
    file_ctx(ctx, "bgms", "bgm", &["mp3"], false)
}

#[export]
fn video(ctx: LineProcessContext) -> LineProcessResult {
    file_ctx(ctx, "videos", "video", &["mp4"], true)
}

#[export]
fn process_action(mut ctx: ActionProcessContext) -> ActionProcessResult {
    let voice_id = ctx.ctx.cur_act.to_string();
    let res = file(
        &voice_id,
        ctx.game_props
            .get("voices")
            .map(|p| AsRef::<Path>::as_ref(p).join(&ctx.ctx.cur_para))
            .as_deref(),
        "voice",
        &["mp3"],
        true,
    );
    ctx.action.vars.extend(res.vars);
    ActionProcessResult { action: ctx.action }
}

#[export]
fn process_game(mut ctx: GameProcessContext) -> GameProcessResult {
    if ctx.props.contains_key("bg") {
        let base_dir = ctx.props.get("bgs").map(AsRef::<Path>::as_ref);
        if let Some(path) = find_exists(&ctx.props["bg"], base_dir, &["png", "jpg", "gif"]) {
            *ctx.props.get_mut("bg").unwrap() = path.to_string_lossy().into_owned();
        }
    }
    GameProcessResult { props: ctx.props }
}
