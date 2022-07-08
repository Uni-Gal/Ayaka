use gal_bindings::*;
use pulldown_cmark::{html::push_html, Parser};

#[export]
fn plugin_type() -> PluginType {
    PluginType::Action
}

#[export]
fn process_action(mut action: ActionData) -> ActionData {
    let parser = Parser::new(&action.line);
    let mut buffer = String::new();
    push_html(&mut buffer, parser);
    action.line = buffer.trim().into();
    action
}
