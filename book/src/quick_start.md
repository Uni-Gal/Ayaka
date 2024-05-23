# Start from source

## Prerequisites
All platforms need [Rust](https://www.rust-lang.org/) and [Nodejs](https://nodejs.org/) installed.
* Rust: nightly toolchain.
* Nodejs: 14.18+/16+ required by [Vite](https://vitejs.dev/).
* `tauri-cli`: `2.0.0-beta.18` ensure latest beta version, required by [Tauri](https://tauri.app/). 
``` bash
$ cargo install tauri-cli --version 2.0.0-beta.18
...
$ cargo tauri --version
tauri-cli 2.0.0-beta.18
```

### Windows
Windows 10+ is recommended but any Windows that Rust supports is OK.

[WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) is required by [Tauri](https://tauri.app/). It comes with the latest Edge browser.

To run the Makefile toolchain, you need GNU Make from [MSYS2](https://www.msys2.org/) project, either for Msys2(`make`) or Mingw64(`mingw32-make`).

Note that if you have a WSL `bash.exe` in PATH before MSYS2 one, the `npm` command may fail.

### Linux
`webkit2gtk` is needed. We only support `webkit2gtk-4.0` required by [Tauri](https://tauri.app/).

### macOS
Generally we don't need anything more, but you should ensure that `make` is installed.

## Clone from source
``` bash
$ git clone https://github.com/Uni-Gal/Ayaka.git
$ cd Ayaka
```

## Add targets for WebAssembly
``` bash
$ rustup target add wasm32-unknown-unknown
```

## Test the utilities
``` bash
$ make test
```

## Run examples
``` bash
$ # Run Fibonacci2
$ make example-Fibonacci2
$ # Run Orga in GUI
$ make example-Orga-gui
```

## Release build of frontends
``` bash
$ make release
```
