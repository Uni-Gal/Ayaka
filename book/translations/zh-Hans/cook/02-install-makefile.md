# 安装 Makefile

Ayaka 使用 Makefile 组织各个组件的编译和测试。

## 如果您正在运行 Windows

一般建议使用 MSYS2 分发的 [`make`](https://packages.msys2.org/package/make)，也可以使用 MSYS2 分发的 [`mingw32-make`](https://packages.msys2.org/base/mingw-w64-make) 或 [chocolatey](https://chocolatey.org/) 分发的 [`make`](https://community.chocolatey.org/packages/make)。

### 安装 MSYS2 分发的 `make`

首先从 [MSYS2 官网](https://www.msys2.org/) 下载安装程序，并按照官网的安装指引安装 MSYS2。

然后在 MSYS2 shell 中，运行
```
pacman -S make
```
安装 `make`。如果想要安装特定的 MinGW 环境的 Makefile，例如 MinGW64 环境，可以运行
```
pacman -S mingw-w64-x86_64-make
```
以安装该环境的 `mingw32-make`。

### 安装 chocolatey 分发的 `make`

首先从 [chocolatey 官网](https://chocolatey.org/install) 根据相应的指引安装 chocolatey。

然后在命令行中，运行
```
choco install make
```

### 将 `make` 加入到环境变量 `PATH`

如果你使用 MSYS2，`make` 是不会被加入到 `PATH` 中的，你只有在使用 MSYS2 shell 的时候才能够调用它。针对这样的情况，你可以选择将 MSYS2 的 `/usr/bin` 目录（或者是 MinGW64 对应的 `/mingw64/bin`）对应的 Windows 目录，一般是 `C:\msys64\usr\bin`（或者 `C:\msys64\mingw64\bin`）加入环境变量的 `PATH` 中。

也可以选择不更改环境变量，而是让 MSYS2 shell 继承 Windows 的环境变量，从而能够调用 `cargo` 和 `npm`。参考[开发者给出的解决方案](https://sourceforge.net/p/msys2/discussion/general/thread/dbe17030/)，可以通过更改环境变量 `MSYS2_PATH_TYPE` 为 `inherit` 来实现。

## 如果您正在运行 Linux

Linux 的发行版各有不同，但是通常包名应该为 `make`。
