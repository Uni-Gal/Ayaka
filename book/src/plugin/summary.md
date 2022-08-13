# Plugin
All plugins should target WebAssembly.
We now support `wasm32-unknown-unknown` and `wasm32-wasi` targets.

The plugin runtime is supported by [Wasmer](https://wasmer.io/).
Our [platform support](../platforms.md) is largely limited by this engine.

We provide a crate `gal-bindings` to easily author a plugin in Rust.

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
