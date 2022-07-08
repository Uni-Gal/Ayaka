use gal_bindings::*;
use pulldown_cmark::{html::push_html, Parser};

#[export]
fn plugin_type() -> PluginType {
    PluginType::Action
}

#[export]
fn process_action(frontend: FrontendType, mut action: ActionData) -> ActionData {
    match frontend {
        FrontendType::Html => {
            let parser = Parser::new(&action.line);
            let mut buffer = String::new();
            push_html(&mut buffer, parser);
            action.line = buffer.trim().into();
        }
        _ => {}
    }
    action
}
