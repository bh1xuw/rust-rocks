# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## Unreleased
### Added
- WriteBatch methods that accept SliceParts
- Add unsafe fn DB::close

### Changed
- Refactor multi get API using PinnableSlice

## 0.1.6
### Added
- New examples
- Options file handling: `load_latest_options`
- Secondary instance support

### Changed
- Use edition 2018
- Link against RocksDB 6.7.3
- New DB interator implementation
- Refactored ColumnFamilyDescriptor
- Rename `Status` to `Error`, refine implementation
- Refactor DBRef using Arc
- Minor argument type changes

## 0.1.5
### Changed
- Use snappy version 1.1.7

### Fixed
- Static link under linux
- Refine feature gates
- Travis CI errors
- Fix create_missing_column_families #5

## 0.1.4
### Added
- persistent_cache.h: add PersistentCache factory method

### Changed
- Link against RocksDB 6.6.4

## 0.1.2 - 2017-08-24
### Added
- convenience.h useful functions, like options stringify
- more usefull functions in env.h
- `Env::get_thread_list` + ThreadStatus support

### Changed
- Now CF handling splits into ColumnFamily and ColumnFamilyHandle

## 0.1.1 - 2017-08-22
### Added
- New Options after RocksDB 5.4 to 5.7.2

### Changed
- Some function now use `P: AsRef<Path>` + `T: IntoIterator<Item=P>` style arguments
- README badges now compatiable with crates.io

### Removed
- Deprecated options by RocksDB 5.7.2

## 0.1.0 - 2017-08-21
### Added
- Event listener API

### Changed
- Fix static link build
- Reformat code with clang-format, 120 col
- Link against RocksDB 5.7.2

## pre-0.1.0
Free-style development. :)
