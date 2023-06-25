#![deny(unsafe_code)]

use ayaka_bindings::{fs::HostFS, vfs::*, *};
use std::collections::{HashMap, HashSet};

#[export]
fn plugin_type() -> PluginType {
    PluginType::builder().line(["show", "hide"]).game().build()
}

fn find_model(ch: &str, game_props: &HashMap<String, String>) -> Option<VfsPath> {
    let root: VfsPath = HostFS.into();
    game_props.get("ch_models").and_then(|ch_models| {
        let base_dir = root.join(ch_models).ok()?;
        ["model.json", "model3.json"]
            .iter()
            .filter_map(|ex| base_dir.join(ch).ok()?.join(format!("{}.{}", ch, ex)).ok())
            .find(|p| p.exists().unwrap_or_default())
    })
}

#[export]
fn show(ctx: LineProcessContext) -> LineProcessResult {
    let models = ctx.props["show"].get_str();
    let exist_models = ctx
        .ctx
        .locals
        .get("ch_models")
        .map(|value| value.get_str())
        .unwrap_or_default();
    let models = exist_models
        .split(',')
        .chain(
            models
                .split(',')
                .filter(|name| ctx.game_props.contains_key(&format!("ch_{}_model", name))),
        )
        .collect::<Vec<_>>();
    let mut res = LineProcessResult::default();
    res.locals
        .insert("ch_models".to_string(), RawValue::Str(models.join(",")));
    res
}

#[export]
fn hide(ctx: LineProcessContext) -> LineProcessResult {
    let hide = ctx.props["hide"].get_str();

    let models = ctx
        .ctx
        .locals
        .get("ch_models")
        .map(|value| value.get_str())
        .unwrap_or_default();
    let models = if hide.is_empty() {
        vec![]
    } else {
        let hide = hide.split(',').collect::<HashSet<_>>();
        models
            .split(',')
            .filter(|ch| !hide.contains(ch))
            .collect::<Vec<_>>()
    };

    let mut res = LineProcessResult::default();
    res.locals
        .insert("ch_models".to_string(), RawValue::Str(models.join(",")));
    res
}

#[export]
fn process_game(mut ctx: GameProcessContext) -> GameProcessResult {
    if let Some(names) = ctx.props.remove("ch_names") {
        for name in names.split(',') {
            if let Some(path) = find_model(name, &ctx.props) {
                ctx.props
                    .insert(format!("ch_{}_model", name), path.as_str().to_string());
            }
        }
    }
    GameProcessResult { props: ctx.props }
}
