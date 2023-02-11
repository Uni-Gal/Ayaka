# 安装Rust环境

Rust是一门现代化的编程语言，且为编译执行，这就意味着并不需要特别的运行环境或解释器才能运行。我们需要安装的是Rust的开发环境。

首先，我们找到Rust语言的[官网](https://www.rust-lang.org/)，官网提供了多种语言的本地化，因此您可以切换点击“English”切换到简体中文界面方便阅读。随后我们依照官网的[安装指南](https://www.rust-lang.org/zh-CN/tools/install)下载。

Rust可在Windows、Linux、macOS、FreeBSD和NetBSD上运行。

## 如果您正在运行Windows

则根据自己电脑的实际情况选择下载[32位](https://static.rust-lang.org/rustup/dist/i686-pc-windows-msvc/rustup-init.exe)或[64位](https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe)的Rustup安装器。

## 如果您正在运行Unix
在终端中运行如下指令，然后按照提示交互安装。
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

下载部分组件可能需要较长的时间，推荐在网络条件（尤其是访问境外服务器）良好的情况下进行。

一切完成后，在终端内输入`rustc --version`，`cargo --version`和`rustup --version`以确认安装正常。

## 换源tuna

https://mirrors.tuna.tsinghua.edu.cn/help/crates.io-index.git/

## 切换到 nightly

```
rustup install nightly
rustup default nightly
```

## 每次在运行前的操作

因为是nightly，所以经常更新
```
rustup update
```
此外还有

```
git pull
```