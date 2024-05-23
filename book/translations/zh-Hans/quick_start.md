# 从源代码开始

## 先决条件
所有平台都需要安装有[Rust](https://www.rust-lang.org/)和[Nodejs](https://nodejs.org/)。
* Rust: nightly toolchain.
* Nodejs: [Vite](https://vitejs.dev/)需要14.18+/16+。
* `tauri-cli`: [Tauri](https://tauri.app/)要求保证是最新的beta版本`2.0.0-beta.18`。
``` bash
$ cargo install tauri-cli --version 2.0.0-beta.18
...
$ cargo tauri --version
tauri-cli 2.0.0-beta.18
```

### Windows
建议使用Windows 10+，但Rust支持的任何Windows版本皆可。

[Tauri](https://tauri.app/)需要[WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)。最新的Edge浏览器已包含它。

要运行Makefile工具链，您需要来自[MSYS2](https://www.msys2.org/)项目的GNU Make，Msys2(`make`)或Mingw64(`mingw32-make`)皆可。

请注意，如果在您的PATH中来自WSL的`bash.exe`位于MSYS2之前，则`npm`命令可能会失败。

### Linux
需要`webkit2gtk`。我们仅支持[Tauri](https://tauri.app/)所依赖的`webkit2gtk-4.0`。

### macOS
通常我们不需要更多依赖，但你应该确保至少有`make`。

## 从源代码Clone
``` bash
$ git clone https://github.com/Uni-Gal/Ayaka.git
$ cd Ayaka
```

## 为WebAssembly添加target
``` bash
$ rustup target add wasm32-unknown-unknown
```

## 进行基础测试
``` bash
$ make test
```

## 运行实例
``` bash
$ # Run Fibonacci2
$ make example-Fibonacci2
$ # Run Orga in GUI
$ make example-Orga-gui
```

## 构建前端
``` bash
$ make release
```
