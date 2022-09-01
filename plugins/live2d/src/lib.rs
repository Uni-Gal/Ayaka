use gal_bindings::*;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

#[export]
fn plugin_type() -> PluginType {
    PluginType::builder()
        .action()
        .text(["show", "hide"])
        .game()
        .build()
}

const CH_DEFAULT: &str = "__ch_default__";
const CH_ALL: &str = "__ch_all__";

fn find_model(
    ch: &str,
    root_path: impl AsRef<Path>,
    game_props: &HashMap<String, String>,
) -> Option<PathBuf> {
    let base_dir = root_path.as_ref().join(
        game_props
            .get("ch_models")
            .map(|s| s.as_str())
            .unwrap_or(""),
    );
    ["model.json", "model3.json"]
        .iter()
        .map(|ex| base_dir.join(ch).join(ch).with_extension(ex))
        .find(|p| p.exists())
}

#[export]
fn show(args: Vec<String>, ctx: TextProcessContext) -> TextProcessResult {
    let models = args
        .into_iter()
        .filter(|name| ctx.game_props.contains_key(&format!("ch_{}_model", name)))
        .collect::<Vec<_>>();
    let mut res = TextProcessResult::default();
    res.props.insert(
        "ch_models".to_string(),
        if models.is_empty() {
            CH_DEFAULT.to_string()
        } else {
            models.join(",")
        },
    );
    res
}

#[export]
fn hide(args: Vec<String>, _ctx: TextProcessContext) -> TextProcessResult {
    let mut res = TextProcessResult::default();
    res.props.insert(
        "ch_hide".to_string(),
        if args.is_empty() {
            CH_ALL.to_string()
        } else {
            args.join(",")
        },
    );
    res
}

#[export]
fn process_action(mut ctx: ActionProcessContext) -> Action {
    let hide = ctx.action.props.remove("ch_hide");
    if hide.as_deref() == Some(CH_ALL) {
        ctx.action.props.remove("ch_models");
    } else {
        let hide = hide
            .as_ref()
            .map(|hide| hide.split(',').collect::<HashSet<_>>())
            .unwrap_or_default();

        let models = ctx
            .action
            .props
            .entry("ch_models".to_string())
            .or_insert_with_key(|key| {
                ctx.last_action
                    .and_then(|act| act.props.get(key).cloned())
                    .unwrap_or_default()
            });

        if models == CH_DEFAULT {
            if let Some(ch) = &ctx.action.ch_key {
                *models = ch.clone()
            }
        }

        *models = models
            .split(',')
            .filter(|name| !hide.contains(name))
            .collect::<Vec<_>>()
            .join(",");
    }
    ctx.action
}

#[export]
fn process_game(mut ctx: GameProcessContext) -> GameProcessResult {
    if let Some(names) = ctx.props.remove("ch_names") {
        for name in names.split(',') {
            if let Some(path) = find_model(name, &ctx.root_path, &ctx.props) {
                ctx.props.insert(
                    format!("ch_{}_model", name),
                    path.to_string_lossy().into_owned(),
                );
            }
        }
    }
    GameProcessResult { props: ctx.props }
}
