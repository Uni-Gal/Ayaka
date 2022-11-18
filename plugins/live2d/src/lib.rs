use ayaka_bindings::*;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

#[export]
fn plugin_type() -> PluginType {
    PluginType::builder().line(["show", "hide"]).game().build()
}

fn find_model(
    ch: &str,
    root_path: impl AsRef<Path>,
    game_props: &HashMap<String, String>,
) -> Option<PathBuf> {
    game_props.get("ch_models").and_then(|ch_models| {
        let base_dir = root_path.as_ref().join(ch_models);
        ["model.json", "model3.json"]
            .iter()
            .map(|ex| base_dir.join(ch).join(ch).with_extension(ex))
            .find(|p| p.exists())
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
            if let Some(path) = find_model(name, "/", &ctx.props) {
                ctx.props.insert(
                    format!("ch_{}_model", name),
                    path.to_string_lossy().into_owned(),
                );
            }
        }
    }
    GameProcessResult { props: ctx.props }
}
