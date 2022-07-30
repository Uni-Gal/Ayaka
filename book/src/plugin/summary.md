# Plugin
All plugins should target WebAssembly.
We now support `wasm32-unknown-unknown` and `wasm32-wasi` targets.

The plugin runtime is supported by [Wasmer](https://wasmer.io/).
Our [platform support](../platforms.md) is largely limited by this engine.

We provide a crate `gal-bindings` to easily author a plugin in Rust.
