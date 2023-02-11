# Install Makefile

Ayaka uses Makefile to organize the compilation and testing of the components.

## If you are working on Windows

We suggest using MSYS2 distributed [`make`](https://packages.msys2.org/package/make) or MSYS2 distributed [`mingw32-make`](https://packages.msys2.org/base/mingw-w64-make), or [chocolatey](https://chocolatey.org/) distributed [`make`](https://community.chocolatey.org/packages/make).

### Install MSYS2 distributed one

Download the installer from [MSYS2 official site](https://www.msys2.org/) and follow the installation instructions.

And run the following command in the MSYS2 shell
```
pacman -S make
```
to install `make`. If you would like a specific MinGW Makefile, e.g. MinGW64 one, run
```
pacman -S mingw-w64-x86_64-make
```
to install `mingw32-make` in that environment.

### Install chocolatey distributed one

Follow the instructions on the [chocolatey official site](https://chocolatey.org/install) to install chocolatey.

Run the following command
```
choco install make
```

### Add `make` to `PATH` environmental variable

If you use MSYS2, `make` won't be add into `PATH`, and you can only use it in the MSYS2 shell. To make it easier, you can choose to add the corresponding directory of MSYS2 `/usr/bin` (or MinGW64 `/mingw64/bin`), usually `C:\msys64\usr\bin` (or `C:\msys64\mingw64\bin`) into environmental variable `PATH`.

You can also choose to not changing the environmental variable, but let MSYS2 shell inherit the variables from Windows, to call `cargo` and `npm` in it. According to the [solution from developers](https://sourceforge.net/p/msys2/discussion/general/thread/dbe17030/), you can change environmental variable `MSYS2_PATH_TYPE` to `inherit`.

## If you are working on Linux

Distros of Linux differ, but usually the package name should be `make`.
