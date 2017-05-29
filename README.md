# RustRocks [![crates.io badge][crates-io-badge]][crates-io-url]

Make RocksDB really rocks!

working in progress.

## status

big picture(TODOs):

- [ ] git submodule, static-link, compression as feature gate
- [x] information hiding (DO NOT EXPORT raw pointers)
- [ ] Proof of usablility
- [ ] bench across C++/Java/other-rust binding
- [ ] Zero-Copy
  - [ ] wraps std::string in Rust
  - [ ] exports String/Vec<u8> to C++ via `assign`-style API
- [ ] Full documentation with code examples
  - [x] good enough by copying C++ comments
  - [ ] rename C++ function names to rust name in doc comments

> checkbox means DONE, or NEEDLESS TO BE DONE.

- [x] ~~``rocksdb/c.h``~~
  - [x] we use c++ API
- [x] ``rocksdb/cache.h``
  - [ ] customized cache support?
- [x] ``rocksdb/compaction_filter.h``
  - [x] use v2 style api, via rust trait
  - [ ] compaction filter factory
- [ ] ``rocksdb/compaction_job_stats.h``
- [x] ``rocksdb/comparator.h``
  - [x] bitwise as new options
  - [x] customized comparator via rust trait
- [ ] ``rocksdb/convenience.h``
- [x] ``rocksdb/db.h``
  - [x] column family support, both in CF/DB
  - [x] open / close
  - [x] iterator
  - [x] get/put/delete
  - [x] MultiGet
  - [x] write
  - [x] merge
  - [x] KeyMayExist - bool return 
  - [x] KeyMayExist - get return
  - [x] properties
    - [x] string/int properties
    - [ ] get map properties
  - [x] compact range
  - [x] flush
  - [x] ingest sst file
  - [ ] compact files
  - [x] misc functions related to config
  - [ ] misc functions related to size approximate
- [ ] ~~``rocksdb/db_bench_tool.h``~~
- [x] ``rocksdb/db_dump_tool.h``
- [ ] ``rocksdb/env.h``
  - [ ] logger
- [ ] ~~``rocksdb/experimental.h``~~
- [x] ``rocksdb/filter_policy.h``
  - [x] basic bloom filter
  - [ ] customized filter policy
- [ ] ``rocksdb/flush_block_policy.h``
- [x] ``rocksdb/iostats_context.h``
- [x] ``rocksdb/iterator.h``
  - [x] adapter for Rust Iterator
  - [ ] reverse iterator
  - [ ] pinned slice, wait rocksdb 5.4
- [ ] ~~``rocksdb/ldb_tool.h``~~
- [ ] ``rocksdb/listener.h``
- [ ] ``rocksdb/memtablerep.h``
- [x] ``rocksdb/merge_operator.h``
  - [x] associative merge operator
  - [x] merge operator - full merge v2
  - [ ] merge operator - full merge - ``assign_existing_operand``
  - [ ] merge operator - partial merge
- [ ] ``rocksdb/metadata.h``
- [x] ``rocksdb/options.h``
  - [x] builder style
- [x] ``rocksdb/perf_context.h``
- [x] ``rocksdb/perf_level.h``
- [ ] ``rocksdb/persistent_cache.h``
- [x] ``rocksdb/rate_limiter.h``
- [x] ~~``rocksdb/slice.h``~~
  - [x] use ``&[u8]`` to replace
- [x] ``rocksdb/slice_transform.h``
- [x] ``rocksdb/snapshot.h``
  - [ ] ManagedSnapshot
- [ ] ``rocksdb/sst_dump_tool.h``
- [ ] ``rocksdb/sst_file_manager.h``
- [x] ``rocksdb/sst_file_writer.h``
- [x] ``rocksdb/statistics.h``
  - [ ] customized statistics class via rust trait?
- [x] ``rocksdb/status.h``
  - [ ] Rust style Error? (i.e. remove Status::OK)
- [x] ``rocksdb/table.h``
  - [x] plain / block based / cuckoo options
  - [ ] customized
- [ ] ``rocksdb/table_properties.h``
- [ ] ``rocksdb/thread_status.h``
- [ ] ``rocksdb/threadpool.h``
- [ ] ``rocksdb/transaction_log.h``
- [x] ``rocksdb/types.h``
  - [x] a sequence number type, wrapped in Snapshot
- [x] ``rocksdb/version.h``
- [ ] ``rocksdb/wal_filter.h``
  - [x] basic trait
- [x] ``rocksdb/write_batch.h``
  - [x] basic functions
  - [x] builder style
  - [x] batch cf ops
  - [x] inspect functions ``has_*``
  - [ ] putv/deletev
  - [ ] handler/iterate
- [ ] ~~``rocksdb/write_batch_base.h``~~
- [ ] ``rocksdb/write_buffer_manager.h``
- [ ] ``rocksdb/universal_compaction.h``
- [ ] ``rocksdb/utilities/backupable_db.h``
- [ ] ``rocksdb/utilities/checkpoint.h``
- [ ] ``rocksdb/utilities/convenience.h``
- [ ] ``rocksdb/utilities/date_tiered_db.h``
- [ ] ``rocksdb/utilities/db_ttl.h``
- [ ] ``rocksdb/utilities/document_db.h``
- [ ] ``rocksdb/utilities/env_librados.h``
- [ ] ``rocksdb/utilities/env_mirror.h``
- [ ] ``rocksdb/utilities/geo_db.h``
- [ ] ``rocksdb/utilities/info_log_finder.h``
- [ ] ``rocksdb/utilities/json_document.h``
- [ ] ``rocksdb/utilities/ldb_cmd.h``
- [ ] ``rocksdb/utilities/ldb_cmd_execute_result.h``
- [ ] ~~``rocksdb/utilities/leveldb_options.h``~~
- [ ] ~~``rocksdb/utilities/lua/rocks_lua_compaction_filter.h``~~
- [ ] ~~``rocksdb/utilities/lua/rocks_lua_custom_library.h``~~
- [ ] ~~``rocksdb/utilities/lua/rocks_lua_util.h``~~
- [ ] ``rocksdb/utilities/memory_util.h``
- [ ] ``rocksdb/utilities/object_registry.h``
- [ ] ``rocksdb/utilities/optimistic_transaction_db.h``
- [ ] ``rocksdb/utilities/option_change_migration.h``
- [ ] ``rocksdb/utilities/options_util.h``
- [ ] ``rocksdb/utilities/sim_cache.h``
- [ ] ``rocksdb/utilities/spatial_db.h``
- [ ] ``rocksdb/utilities/stackable_db.h``
- [ ] ``rocksdb/utilities/table_properties_collectors.h``
- [ ] ``rocksdb/utilities/transaction.h``
- [ ] ``rocksdb/utilities/transaction_db.h``
- [ ] ``rocksdb/utilities/transaction_db_mutex.h``
- [ ] ~~``rocksdb/utilities/utility_db.h``~~
- [ ] ``rocksdb/utilities/write_batch_with_index.h``

[crates-io-badge]: https://img.shields.io/crates/v/rocks.svg
[crates-io-url]: https://crates.io/crates/rocks
