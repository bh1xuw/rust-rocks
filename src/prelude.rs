//! The `RocksDB` prelude entry.

use rocks_sys as ll;

pub use self::ll::version;

pub use crate::comparator::Comparator;
pub use crate::db::*;
pub use crate::env::{Env, Logger};
pub use crate::merge_operator::{AssociativeMergeOperator, MergeOperator};
pub use crate::options::*;
pub use crate::perf_level::*;
pub use crate::slice::{CVec, PinnableSlice};
pub use crate::table::*;
pub use crate::table_properties::{TableProperties, TablePropertiesCollection};
pub use crate::transaction_log::LogFile;
pub use crate::types::SequenceNumber;
pub use crate::write_batch::WriteBatch;

pub use super::Error;

#[test]
fn test_version() {
    let v = version();
    println!("version = {}", v);
    assert!(v >= "5.3.1".into());
}
