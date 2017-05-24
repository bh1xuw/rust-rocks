//! Table options.
//!
//! Currently we support two types of tables: plain table and block-based table.
//! 1. Block-based table: this is the default table type that we inherited from
//!    LevelDB, which was designed for storing data in hard disk or flash
//!    device.
//! 2. Plain table: it is one of RocksDB's SST file format optimized
//!    for low query latency on pure-memory or really low-latency media.
//!
//! A tutorial of rocksdb table formats is available here:
//! >  https://!github.com/facebook/rocksdb/wiki/A-Tutorial-of-RocksDB-SST-formats
//!
//! Example code is also available
//! > https://!github.com/facebook/rocksdb/wiki/A-Tutorial-of-RocksDB-SST-formats#wiki-examples

pub struct TableFactory;
