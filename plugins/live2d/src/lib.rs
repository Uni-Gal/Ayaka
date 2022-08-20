use gal_bindings::*;

#[export]
fn plugin_type() -> PluginType {
    PluginType::ACTION
}

#[export]
fn process_action(mut ctx: ActionProcessContext) -> Action {
    if let Some(ch) = &ctx.action.ch_key {
        let base_dir = ctx.root_path.join(
            ctx.game_props
                .get("models")
                .map(|s| s.as_str())
                .unwrap_or(""),
        );
        let model_path = ["model.json", "model3.json"]
            .iter()
            .map(|ex| base_dir.join(ch).join(ch).with_extension(ex))
            .find(|p| p.exists());
        if let Some(path) = model_path {
            ctx.action
                .props
                .insert("ch_model".to_string(), path.to_string_lossy().into_owned());
        }
    }
    ctx.action
}
