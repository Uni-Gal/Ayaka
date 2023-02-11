# Install Rust

Rust is a modern, compiled language, which means it doesn't need a special runtime environment or an interpreter to execute. What we need to install is the development environment of Rust.

Browse to the [official cite](https://www.rust-lang.org/) of Rust first. Then we download following the [installation guide](https://www.rust-lang.org/tools/install) on the official cite.

Rust can run on Windows, Linux, macOS, FreeBSD and NetBSD.

## If you are working on Windows

Choose to download [32-bit](https://static.rust-lang.org/rustup/dist/i686-pc-windows-msvc/rustup-init.exe) or [64-bit](https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe) `rustup` installer according your operating system.

## If you are working on Unix

Run the following command in the terminal
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
and follow the instructions.

After the installation done, type `rust --version`, `cargo --version` and `rustup --version` in the terminal to make sure the installation valid.

## Switch to nightly

```
rustup install nightly
rustup default nightly
```

## Notes on every time before running...

We use nightly toolchain to compile. You need to update regularly
```
rustup update
```
and
```
git pull
```
