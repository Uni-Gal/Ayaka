use gal_bindings::*;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[export]
fn plugin_type() -> PluginType {
    PluginType::TEXT | PluginType::GAME
}

#[export]
fn text_commands() -> &'static [&'static str] {
    &["show"]
}

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
    res.props
        .insert("ch_models_count".to_string(), models.len().to_string());
    for (i, name) in models.into_iter().enumerate() {
        res.props.insert(format!("ch_model_{}", i), name);
    }
    res
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
