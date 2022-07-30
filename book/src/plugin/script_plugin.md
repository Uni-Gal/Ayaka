# Script plugin
All plugins are script plugins.
[Scripts](../config/script.md) could call methods in the script plugins.

## Calling methods
A plugin method is referenced with `<module>.<fn>(...)` grammar.
If you would like to call the `rnd` function in `random` module,
``` yaml
- \exec{random.rnd()}
```
Pass the parameters in the brace `()`.

As the scripts are calculated at runtime, if there's no plugin called `random`,
or no method called `rnd` inside `random`, it will give a warning, and continue with `RawValue::Unit`.

## Author a script plugin
Here we're going to author a script plugin `meet` to return a string "Hello".
``` rust,ignore
use gal_bindings::*;

#[export]
fn plugin_type() -> PluginType {
    PluginType::SCRIPT
}

#[export]
fn hello(_args: Vec<RawValue>) -> RawValue {
    RawValue::Str("Hello".to_string())
}
```
Compile this lib with target `wasm32-unknown-unknown`, and you can now load this new plugin in the config file:
``` yaml
plugins:
  dir: path/to/plugins
  modules:
    - meet
```
And call the function:
``` yaml
- \exec{meet.hello()} from plugin!
```
If it builds successfully, and you set the right path to the plugins, it will output:
``` ignore
Hello from plugin!
```
