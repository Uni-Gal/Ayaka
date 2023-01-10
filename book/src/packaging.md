# Packaging
We support package format called `ayapack`.
Internally it is a TAR format archive, without any compression.
We load the package file with memory map to reduce allocations.

## Packaging
`tar` executable is pre-installed on almost all platforms. There may be BSD-Tar or GNU-Tar. For a directory `foo` that contains `config.yaml`, we can simply execute

``` bash
$ cd foo
$ tar -cf foo.ayapack *
```

The parameter `c` means creating a package, and `f` means the following parameter is the package path.

## Details
The details of the parsing and loading are in the [`vfs-tar`](https://github.com/Berrysoft/vfs-tar).
