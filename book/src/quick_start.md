# Quick start

## Prerequisites
All platforms need [Rust](https://www.rust-lang.org/) and [Nodejs](https://nodejs.org/) installed.
* Rust: nightly toolchain.
* Nodejs: 14.18+/16+ required by [Vite](https://vitejs.dev/).

### Windows
Windows 10 1903+ is required because we need `icu.dll`. We may consider extend this support to 1703+(with `icuuc.dll`).

[WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) is required by [Tauri](https://tauri.app/). It comes with the latest Edge browser.

To run the Makefile toolchain, you need GNU Make from [MSYS2](https://www.msys2.org/) project, either for Msys2(`make`) or Mingw64(`mingw32-make`).

Note that if you have a WSL `bash.exe` in PATH before MSYS2 one, the `npm` command may fail.

### Linux
`libicu` and `webkit2gtk` is needed. We only support `webkit2gtk-4.0` required by [Tauri](https://tauri.app/).

### macOS
Generally we don't need anything more, but you should ensure there a `make`.

## Clone from source
``` bash
$ git clone https://github.com/Berrysoft/gal.git
$ cd gal
```

## Test the utilities
``` bash
$ make test
```

## Release build of frontends
``` bash
$ make release
```

## Run examples
``` bash
$ # Run Fibonacci2
$ make example-Fibonacci2
$ # Run Orga in GUI
$ make example-Orga-gui
```
