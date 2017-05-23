#![allow(unused_variables, unused_imports, dead_code)]

extern crate rocks_sys;

#[cfg(test)]
extern crate tempdir;

pub use status::Status;

pub mod advanced_options;
pub mod cache;
pub mod compaction_filter;
pub mod comparator;
pub mod db;
pub mod env;
pub mod iterator;
pub mod listener;
pub mod merge_operator;
pub mod options;
pub mod rate_limiter;
pub mod slice_transform;
pub mod snapshot;
pub mod sst_file_manager;
pub mod sst_file_writer;
pub mod statistics;
pub mod status;
pub mod table;
pub mod table_properties;
pub mod types;
pub mod universal_compaction;
pub mod write_batch;
pub mod write_buffer_manager;
pub mod metadata;

pub mod rocksdb;


#[no_mangle]
pub extern "C" fn rust_hello_world() {
    println!("Hello World! from rust");
}
