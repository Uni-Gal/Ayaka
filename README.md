<img width=100 src="assets/logo.png"/>

# Ayaka

Ayaka is currently a project for [OSPP 2022](https://summer-ospp.ac.cn/).

For Simplified Chinese version README, see [简体中文](https://github.com/Uni-Gal/Ayaka/blob/master/README_zh-Hans.md)

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
* A GUI frontend powered by [Tauri](https://tauri.app/) and [Vue](https://vuejs.org/), with [Live2D](https://www.live2d.com) support.
* A prototype LaTeX frontend to generate PDF from the config.

## Docs
[Ayaka Book](https://uni-gal.github.io/Ayaka/).

For API docs, build into `utils/target/doc`:
``` bash
$ make doc
```

## Screenshot
![Orga](assets/galgui.png)

## License

This project is licensed under the [MIT license](LICENSE).
