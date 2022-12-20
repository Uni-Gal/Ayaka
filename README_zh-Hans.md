<img width=100 src="assets/logo.png"/>

# Ayaka

Ayaka目前是[OSPP 2022](https://summer-ospp.ac.cn/)的一个项目。

## 关于名字
前端是 Ayaka。运行时是 Ayaka。脚本是 Ayaka。 [Just Ayaka.](https://bbs.mihoyo.com/ys/article/21828380)

## 目前已完成的
* 使用Rust完成的跨平台视觉小说引擎。
* 基于 YAML 的定义明确且易于创作的视觉小说配置文件格式。
* 嵌入式的自定义脚本。
* 基于[CLDR](https://github.com/unicode-org/cldr) i18n支持。
* 基于 [WebAssembly](https://webassembly.org/) 的一个灵活的插件系统。它通过脚本提供运行时和互操作功能的钩子。
* 一个“解耦合”的框架——前端、后端以及插件互相解耦合。
* 用于检查语法错误和快速调试的 CLI 前端。
* 基于[Tauri](https://tauri.app/) 与 [Vue](https://vuejs.org/) 的GUI前端，并包含 [Live2D](https://www.live2d.com) 支持。
* 从配置直接生成 PDF 的 LaTeX 前端原型。

## 文档
对于不够擅长编程的创作者，可见 [Ayaka cookbook](https://github.com/Uni-Gal/Ayaka-cookbook).

对于开发者，可见 [Ayaka Book](https://uni-gal.github.io/Ayaka/).

API文档需要在 `utils/target/doc`内构建：
``` bash
$ make doc
```

## 运行截屏
![奥尔加](assets/galgui.png)

## 许可证

项目依[MIT license](LICENSE)许可。
