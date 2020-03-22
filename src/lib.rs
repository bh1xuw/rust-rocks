//! Rust-Rocks
//!
//! The RocksDB API in Rustic Style.
//!
//! # Examples
//!
//! ```no_run
//! use rocks::rocksdb::*;
//! // RAII DB instance
//! let db = DB::open(&Options::default(), "./data").unwrap();
//! assert!(db.put(&WriteOptions::default(), b"my key", b"my value").is_ok());
//! match db.get(&ReadOptions::default(), b"my key") {
//!     Ok(ref value) => println!("retrieved value {}", String::from_utf8_lossy(value)),
//!     Err(e) => println!("operational problem encountered: {}", e),
//! }
//! let _ = db.delete(&WriteOptions::default(), b"my key").unwrap();
//! ```
// #![cfg_attr(feature = "dev", feature(plugin))]
// #![cfg_attr(feature = "dev", plugin(clippy))]
// #![cfg_attr(not(feature = "dev"), allow(unknown_lints))]
// #![allow(not_unsafe_ptr_arg_deref, wrong_self_convention, doc_markdown)]
#![allow(unused_variables, dead_code)]

pub use error::Status;

/// The result type returned by RocksDB, wraps Status
pub type Result<T> = std::result::Result<T, Status>;

pub mod advanced_options;
pub mod cache;
pub mod compaction_filter;
pub mod compaction_job_stats;
pub mod comparator;
pub mod convenience;
pub mod db;
pub mod db_dump_tool;
pub mod debug;
pub mod env;
pub mod error;
pub mod filter_policy;
pub mod flush_block_policy;
pub mod iostats_context;
pub mod iterator;
pub mod listener;
pub mod merge_operator;
pub mod metadata;
pub mod options;
pub mod perf_context;
pub mod perf_level;
pub mod persistent_cache;
pub mod rate_limiter;
pub mod slice;
pub mod slice_transform;
pub mod snapshot;
pub mod sst_file_manager;
pub mod sst_file_writer;
pub mod statistics;
pub mod table;
pub mod table_properties;
pub mod thread_status;
pub mod transaction_log;
pub mod types;
pub mod universal_compaction;
pub mod wal_filter;
pub mod write_batch;
pub mod write_buffer_manager;

// the prelude
pub mod rocksdb;

// for raw pointer infomation hiding
mod to_raw;
