# Action plugin
Action plugins deal with the whole action after the text being parsed.
An action plugin usually modifies the text, for example, parses the Markdown text and output to HTML.

Action plugins could access the last action in the history, will text plugins cannot.

## Simple Markdown plugin
Action plugins don't need to register anything, but only to specify in the `plugin_type`:

``` rust,ignore
use gal_bindings::*;
use pulldown_cmark::*;

#[export]
fn plugin_type() -> PluginType {
    PluginType::ACTION
}

#[export]
fn process_action(mut ctx: ActionProcessContext) -> Action {
    let line = ctx
        .action
        .line
        .into_iter()
        .map(|s| s.into_string())
        .collect::<Vec<_>>()
        .concat();
    let parser = Parser::new(&line);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    ctx.action.line.clear();
    ctx.action.line.push_back_chars(html_output);
    ctx.action
}
```

Add the plugin to the `modules` list, and all markdown text will be translated to HTML.

You may notice that the HTML tags are treated as `ActionLine::Chars`, which means they will be displayed one by one on GUI frontends. Our existing `markdown` plugin resolves this problem by providing a custom writer.

## Existing plugins
| Plugin     | Description                                      |
| ---------- | ------------------------------------------------ |
| `markdown` | Process Markdown texts.                          |
| `media`    | Inherit background image and music from history. |
