use std::path::{Path, PathBuf};

use ayaka_bindings::*;

#[export]
fn plugin_type() -> PluginType {
    PluginType::builder()
        .line(["bg", "bgm", "efm", "video"])
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

fn file(arg: &str, base_dir: Option<&Path>, prop: &str, exs: &[&str]) -> LineProcessResult {
    log::debug!(
        "File {:?}, {}, {:?}",
        base_dir.map(|p| p.display()),
        prop,
        exs
    );
    let mut res = LineProcessResult::default();
    if let Some(path) = find_exists(arg, base_dir, exs) {
        res.locals.insert(
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
) -> LineProcessResult {
    file(
        &ctx.props[prop].get_str(),
        ctx.game_props
            .get(game_prop)
            .map(|game_prop| ctx.root_path.join(game_prop))
            .as_deref(),
        prop,
        exs,
    )
}

#[export]
fn bg(ctx: LineProcessContext) -> LineProcessResult {
    file_ctx(ctx, "bgs", "bg", &["png", "jpg", "gif"])
}

#[export]
fn bgm(ctx: LineProcessContext) -> LineProcessResult {
    file_ctx(ctx, "bgms", "bgm", &["mp3"])
}

#[export]
fn efm(ctx: LineProcessContext) -> LineProcessResult {
    file_ctx(ctx, "efms", "efm", &["mp3"])
}

#[export]
fn video(ctx: LineProcessContext) -> LineProcessResult {
    file_ctx(ctx, "videos", "video", &["mp4"])
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
