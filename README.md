# Ayaka

Ayaka is currently a project for [OSPP 2022](https://summer-ospp.ac.cn/).

## About the name
The frontend is Ayaka. The runtime is Ayaka. The script is Ayaka. [Just Ayaka.](https://bbs.mihoyo.com/ys/article/21828380)

## What we've done
* A cross-platform visual noval (VN) runtime with rust
* A well-defined and easy-to-author VN config file format, based on YAML.
* An embedded custom script.
* [CLDR](https://github.com/unicode-org/cldr)-based i18n support.
* A flexible plugin system based on [WebAssembly](https://webassembly.org/). It provides hooks of the runtime and interop functionalities with the script.
* A decoupled framework - the frontend, backend and plugins are decoupled.
* A CLI frontend to check grammar errors and debug quickly.
* A GUI frontend powered by [Tauri](https://tauri.app/) and [Vue](https://vuejs.org/).
* A proto LaTeX frontend to generate PDF from the config.

## Docs
See Ayaka Book:
``` bash
$ make serve-book
```
If you don't have `mdbook` installed, simply read the markdown files in `book` folder.
To start quickly, follow the instructions in [Quick start](./book/src/quick_start.md).

Build API docs into `utils/target/doc`:
``` bash
$ make doc
```

## Screenshot
![Orga](assets/galgui.png)
