# 支持的平台
Ayaka 跨平台，并将支持的平台（三元组）分为三级。

## 第一级
保证工作：

* x86_64-pc-windows-msvc
* x86_64-unknown-linux-gnu
* x86_64-apple-darwin

## 第二级
应当正常工作，但是没有测试过：

* i686-pc-windows-msvc
* aarch64-pc-windows-msvc
* i686-pc-windows-gnu
* x86_64-pc-windows-gnu
* aarch64-unknown-linux-gnu
* aarch64-apple-darwin

## 第三级
由于依赖可能存在问题，不一定正确编译运行：

* s390x-unknown-linux-gnu
