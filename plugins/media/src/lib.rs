#![deny(unsafe_code)]

use ayaka_bindings::{fs::HostFS, vfs::*, *};

#[export]
fn plugin_type() -> PluginType {
    PluginType::builder()
        .action()
        .line(["bg", "bgm", "video"])
        .game()
        .build()
}

fn find_exists(name: &str, base_dir: Option<&VfsPath>, exs: &[&str]) -> Option<VfsPath> {
    base_dir.and_then(|base_dir| {
        exs.iter()
            .map(|ex| base_dir.join(format!("{}.{}", name, ex)).unwrap())
            .find(|p| p.exists().unwrap_or_default())
    })
}

fn file(
    arg: &str,
    base_dir: Option<&VfsPath>,
    prop: &str,
    exs: &[&str],
    temp: bool,
) -> LineProcessResult {
    log::debug!(
        "File {}, {:?}, {}, {:?}",
        arg,
        base_dir.map(|p| p.as_str()),
        prop,
        exs
    );
    let mut res = LineProcessResult::default();
    if let Some(path) = find_exists(arg, base_dir, exs) {
        if temp { &mut res.vars } else { &mut res.locals }
            .insert(prop.to_string(), RawValue::Str(path.as_str().to_string()));
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
    let root: VfsPath = HostFS::default().into();
    let base_dir = ctx.game_props.get(game_prop).map(|p| root.join(p).unwrap());
    file(
        &ctx.props[prop].get_str(),
        base_dir.as_ref(),
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
    let root: VfsPath = HostFS::default().into();
    let voice_id = ctx.ctx.cur_act.to_string();
    let res = file(
        &voice_id,
        ctx.game_props
            .get("voices")
            .map(|p| root.join(p).unwrap().join(&ctx.ctx.cur_para).unwrap())
            .as_ref(),
        "voice",
        &["mp3"],
        true,
    );
    ctx.action.vars.extend(res.vars);
    ActionProcessResult { action: ctx.action }
}

#[export]
fn process_game(mut ctx: GameProcessContext) -> GameProcessResult {
    let root: VfsPath = HostFS::default().into();
    if ctx.props.contains_key("bg") {
        let base_dir = ctx.props.get("bgs").map(|p| root.join(p).unwrap());
        if let Some(path) = find_exists(&ctx.props["bg"], base_dir.as_ref(), &["png", "jpg", "gif"])
        {
            *ctx.props.get_mut("bg").unwrap() = path.as_str().to_string();
        }
    }
    GameProcessResult { props: ctx.props }
}
