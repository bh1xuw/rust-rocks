# RustRocks

[![Build Status](https://travis-ci.org/bh1xuw/rust-rocks.svg?branch=master)](https://travis-ci.org/bh1xuw/rust-rocks)
[![crates.io badge](https://img.shields.io/crates/v/rocks.svg)](https://crates.io/crates/rocks)

Another RocksDB binding for Rust. [Documentation](https://docs.rs/rocks/)

Make RocksDB really rocks!

## How to compile

Link against: RocksDB 5.7.2.

tests pass under:

- macOS 10.12
- Linux amd64
- Linux aarch64(Odroid-C2)

For macOS(with rocksdb installed via brew):

    LIBRARY_PATH=/usr/local/lib CXXFLAGS=-I/usr/local/include \
    cargo test -- --nocapture

For Linux(with rocksdb installed into /usr/local):

    LD_LIBRARY_PATH=/usr/local/lib \
    LIBRARY_PATH=/usr/local/lib CXXFLAGS=-I/usr/local/include \
    cargo test -- --nocapture

For static build:

    # refer .travis.yml

List all supported compression types:

    cargo test -- --nocapture compression_types

## Installation

```toml
[dependencies]
rocks = "0.1.0"
```

With all static features

```toml
[dependencies.rocks]
version = "0.1.0"
default-features = false
features = ["static-link", "rocks-sys/snappy", "rocks-sys/zlib", "rocks-sys/bzip2", "rocks-sys/lz4", "rocks-sys/zstd"]
```

## TODOs

Big picture:

- [x] git submodule, static-link, compression as feature gate
- [x] information hiding (DO NOT EXPORT raw pointers)
- [x] Rust style
  - [x] wraps Status into a Rust style ``Result<T>``
  - [x] ``*Options`` via builder pattern
  - [ ] handle CFHandle lifetime, Ref safety
- [ ] Lifetime safely guarantee
  - [x] `ReadOptions` + `snapshot`
  - [x] `ReadOptions` + `iterate_upper_bound`
  - [x] `DB` + `ColumnFamilyHandle`
  - [ ] `ColumnFamilyOptions` + `compaction_filter`
  - [ ] `ColumnFamilyOptions` + customized `comparator`
- [ ] Proof of usablility
- [ ] bench across C++/Java/other-rust binding
- [x] CI
  - [x] travis-ci integration
  - [ ] appveyor integration for windows
- [x] Zero-Copy between C++ part
  - [x] pinnable slice support
  - [x] exports String/Vec<u8> to C++ via `assign`-style API
- [ ] Full documentation with code examples
  - [x] good enough by copying C++ comments
  - [ ] rename C++ function names to rust name in doc comments
  - [ ] more examples in doc comment
