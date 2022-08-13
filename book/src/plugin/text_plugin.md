# Text plugin
Text plugins deal with custom TeX-like commands.
They should register the commands in `text_commands` method.

## Register commands
Here we will register a command `\hello` and call it in the config file.
You also need `plugin_type` to specify that it is a text plugin.
``` rust,ignore
use gal_bindings::*;

#[export]
fn plugin_type() -> PluginType {
    PluginType::TEXT
}

#[export]
fn text_commands() -> &'static [&'static str] {
    ["hello"]
}

#[export]
fn hello(_args: Vec<String>, _ctx: TextProcessContext) -> TextProcessResult {
    let mut res = TextProcessResult::default();
    res.line.push_back_chars("Hello");
    res
}
```

Call the command in the config file:
``` yaml
- \hello world!
```
And it outputs:
``` ignore
Hello world!
```

## The process result
The `TextProcessResult` object is some lines and properties to be added to the current action. `line` will be appended to the current position of the command, and `props` will be set and update.

## Existing plugins
| Plugin     | Description          |
| ---------- | -------------------- |
| `basictex` | Basic TeX commands.  |
| `media`    | Multimedia commands. |
