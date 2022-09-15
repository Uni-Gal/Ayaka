# Plugin
All plugins should target WebAssembly.
We now support `wasm32-unknown-unknown` and `wasm32-wasi` targets.

The plugin runtime is supported by [Wasmer](https://wasmer.io/).
Our [platform support](../platforms.md) is largely limited by this engine.

We provide a crate `ayaka-bindings` to easily author a plugin in Rust.

## Load plugins
Specify the plugin directory in the config file:
``` yaml
plugins:
  dir: path/to/plugins
```
The runtime will try to load all WebAssembly file in the directory.
If you want to specify some of them, or specify the load order, specify them in `modules`:
``` yaml
plugins:
  dir: path/to/plugins
  modules:
    - foo
    - bar
```
You don't need to specify the extension.

## The text processing workflow
``` dot process
digraph {
  ori[label="Raw text"];
  lines[label="Structural lines"];
  exec[label="Run scripts"];
  cmd[label="Custom commands"];
  texts[label="Final texts"];
  history[label="Record history"];
  output[label="Output"];

  ori -> lines [label="TextParser"];
  lines -> exec [label="ProgramParser"];
  exec -> cmd [label="text plugins"];
  cmd -> texts [label="action plugins"];
  texts -> history;
  history -> output;
}
```
