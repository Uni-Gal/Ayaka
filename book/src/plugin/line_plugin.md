# Line plugin
Line plugins provides custom line types.

## Register commands
Here we will register a command `\hello` and call it in the config file.
You also need `plugin_type` to specify that it is a line plugin.
``` rust,ignore
use ayaka_bindings::*;

#[export]
fn plugin_type() -> PluginType {
    PluginType::builder().line(["hello"]).build()
}

#[export]
fn hello(_ctx: LineProcessContext) -> LineProcessResult {
    let mut res = LineProcessResult::default();
    res.locals.insert("hello".to_string(), RawValue::Str("Hello".to_string()));
    res
}
```

Call the command in the config file:
``` yaml
- hello:
- \var{hello} world!
```
And it outputs:
``` ignore
Hello world!
```

## The process results
The `LineProcessResult` object contains the global variables and temp variables. The temp variables will only apply to this specific line.

## Existing plugins
| Plugin  | Description          |
| ------- | -------------------- |
| `media` | Multimedia commands. |
