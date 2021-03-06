# RustRocks

[![crates.io badge](https://img.shields.io/crates/v/rocks.svg)](https://crates.io/crates/rocks)
[![DOCS.RS badge](https://docs.rs/rocks/badge.svg)](https://docs.rs/rocks)
[![Linux Build Status](https://travis-ci.org/bh1xuw/rust-rocks.svg?branch=master)](https://travis-ci.org/bh1xuw/rust-rocks)
[![macOS Build Status](https://github.com/bh1xuw/rust-rocks/workflows/Rust/badge.svg)](https://github.com/bh1xuw/rust-rocks/actions)
[![Windows Build Status](https://ci.appveyor.com/api/projects/status/atawb7u8ekseodhm/branch/master?svg=true)](https://ci.appveyor.com/project/bh1xuw/rust-rocks)

Another RocksDB binding for Rust. [Documentation](https://docs.rs/rocks/)

Make RocksDB really rocks!

- Static link against RocksDB 6.7.3 (git submodules)
- Dynamic link tested:
  - macOS homebrew
  - Windows 10, VS 2019 with `vcpkg`
  - ArchLinux pacman, both x86_64 and aarch64(ODroid-C2)
  - Ubuntu 18.04 (`rocksdb5.8` branch), both x86_64 and aarch64(RPi 3)
  - Ubuntu 20.04 (`rocksdb5.17` branch)

## Installation

Dynamicly link RocksDB:

```toml
[dependencies]
rocks = "0.1"
```

Static link against RocksDB(with snappy enabled by default):

```toml
[dependencies.rocks]
version = "0.1"
default-features = false
features = ["static-link"]
```

With all static features(all compression types):

```toml
[dependencies.rocks]
version = "0.1"
default-features = false
features = ["full"]
```

## How to compile

Feel free to refer Travic-CI, AppVeyor and Github Actions configuration files.

### Static Link

```console
$ git submodule update --init --recursive
$ cargo test --features static-link -- --test-threads 1
(This will build with snappy as the only compression supported)

$ cargo test --features full -- --test-threads 1
(This will build with all compression supported)
```

### Dynamic Link

For macOS(with RocksDB installed via brew):

```console
$ brew install rocksdb
$ cargo test -- --nocapture --test-threads 1
```

For Linux:

```console
(install rocksdb via package manager or make & install)
$ sudo apt install lld
(NOTE: gcc-ld can't handle circular references while linking.)
(for more, refer the last section of README)
$ RUSTFLAGS="-C link-arg=-fuse-ld=lld" cargo test -- --test-threads 1
```

Use environment variables if rocksdb is installed to non-default directory:

`LD_LIBRARY_PATH=/usr/local/lib LIBRARY_PATH=/usr/local/lib CXXFLAGS=-I/usr/local/include`

### Ubuntu LTS

RocksDB changes its API often, so `rust-rocks` use different branch to support Ubuntu LTS.

```console
> sudo apt install librocksdb-dev libsnappy-dev
```

You also need `lld` form official source or `http://apt.llvm.org/`.

Branches:

- rocksdb5.8 (18.04 LTS)
- rocksdb5.17 (20.04 LTS)

### Windows

You need VS 2017 or VS 2019, and [install RocksDB](https://github.com/facebook/rocksdb/wiki/Building-on-Windows)
via [vcpkg](https://github.com/Microsoft/vcpkg).

## FAQ

- [Which features are supported/missing comparing to C++ RocksDB?](https://github.com/bh1xuw/rust-rocks/issues/1)
- [Why another RocksDB binding for Rust?](https://github.com/bh1xuw/rust-rocks/issues/2)

Feel free to Open a [New Issue](https://github.com/bh1xuw/rust-rocks/issues/new).

### List current supported compression types

```console
$ cargo run --example it-works
RocksDB: 6.7.3
Compression Supported:
  - NoCompression
  - SnappyCompression
  - ZlibCompression
  - BZip2Compression
  - LZ4Compression
  - LZ4HCCompression
  - ZSTD
  - ZSTDNotFinalCompression
```

## Development

Bindgen:

```console
$ cd rocks-sys
$ PATH="/usr/local/opt/llvm/bin:$PATH" make
(this will regenerate the bindgen c.rs)
```

## Known bugs

### Linking error under Linux

- rust-rocks exports rust functions to c++, so there are circular references while linking
- GCC requires that you put the object files and libraries in the order that they depend on each other
- Rust will not wrap user crates in `--start-group` and `--end-group`
- So circular references will be errors.
- Can be fixed by using `lld` as linker, `RUSTFLAGS="-C link-arg=-fuse-ld=lld"`
- Can be fixed by manually organising link arguments
  - librocks, then librocks_sys, *then librocks again*

### Minor memory leaks

- The raw pointers are created on the fly, should be impled via `lazy_static` and wrapped in trait objects
  - `ColumnFamilyOptions::comparator`: `const Comparator*`
  - `ColumnFamilyOptions::compaction_filter`: `const CompactionFilter*`

### Iterator leaks lifetime

Ref: <https://github.com/bh1xuw/rust-rocks/issues/15>

- While doing a `for`-traverse: That's OK
- While collecting for later use: Clone the keys and values
