//! Rust-Rocks
//!
//! The RocksDB API in Rustic Style.
//!
//! # Examples
//!
//! ```no_run
//! use rocks::rocksdb::*;
//!
//! let opt = Options::default().map_db_options(|db_opt| db_opt.create_if_missing(true));
//! let db = DB::open(opt, "./data").unwrap();
//!
//! assert!(db.put(WriteOptions::default_instance(), b"hello", b"world").is_ok());
//!
//! match db.get(ReadOptions::default_instance(), b"hello") {
//!     Ok(ref value) => println!("hello: {:?}", value),
//!     Err(e) => eprintln!("error: {}", e),
//! }
//! let _ = db.delete(WriteOptions::default_instance(), b"hello").unwrap();
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
pub mod prelude;

// #[deprecated(since = "0.1.5", note = "Please use the `prelude` module instead")]
pub mod rocksdb {
    pub use crate::prelude::*;
}

// for raw pointer infomation hiding
mod to_raw;
