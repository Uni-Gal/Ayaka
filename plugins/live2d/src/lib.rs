use gal_bindings::*;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[export]
fn plugin_type() -> PluginType {
    PluginType::TEXT
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
        .filter_map(|ch| find_model(&ch, &ctx.root_path, &ctx.game_props).map(|path| (ch, path)))
        .collect::<Vec<_>>();
    let mut res = TextProcessResult::default();
    res.props
        .insert("ch_models_count".to_string(), models.len().to_string());
    for (i, (name, m)) in models.into_iter().enumerate() {
        res.props
            .insert(format!("ch_model_{}", i), m.to_string_lossy().into_owned());
        res.props.insert(format!("ch_model_{}_name", i), name);
    }
    res
}
