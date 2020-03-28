//! `CompactionFilter` allows an application to modify/delete a key-value at
//! the time of compaction.

use std::os::raw::{c_char, c_int};

use rocks_sys as ll;

#[repr(C)]
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Copy, Clone)]
pub enum ValueType {
    Value = 0,
    MergeOperand = 1,
}

#[derive(Debug)]
pub enum Decision {
    Keep,
    Remove,
    ChangeValue(Vec<u8>),
    RemoveAndSkipUntil(Vec<u8>),
}

impl Decision {
    // to C Decision type
    fn to_c(&self) -> c_int {
        match *self {
            Decision::Keep => 0,
            Decision::Remove => 1,
            Decision::ChangeValue(_) => 2,
            Decision::RemoveAndSkipUntil(_) => 3,
        }
    }
}

/// `CompactionFilter` allows an application to modify/delete a key-value at
/// the time of compaction.
pub trait CompactionFilter {
    // The compaction process invokes this
    // method for kv that is being compacted. A return value
    // of false indicates that the kv should be preserved in the
    // output of this compaction run and a return value of true
    // indicates that this key-value should be removed from the
    // output of the compaction.  The application can inspect
    // the existing value of the key and make decision based on it.
    //
    // Key-Values that are results of merge operation during compaction are not
    // passed into this function. Currently, when you have a mix of Put()s and
    // Merge()s on a same key, we only guarantee to process the merge operands
    // through the compaction filters. Put()s might be processed, or might not.
    //
    // When the value is to be preserved, the application has the option
    // to modify the existing_value and pass it back through new_value.
    // value_changed needs to be set to true in this case.
    //
    // If you use snapshot feature of RocksDB (i.e. call GetSnapshot() API on a
    // DB* object), CompactionFilter might not be very useful for you. Due to
    // guarantees we need to maintain, compaction process will not call Filter()
    // on any keys that were written before the latest snapshot. In other words,
    // compaction will only call Filter() on keys written after your most recent
    // call to GetSnapshot(). In most cases, Filter() will not be called very
    // often. This is something we're fixing. See the discussion at:
    // https://www.facebook.com/groups/mysqlonrocksdb/permalink/999723240091865/
    //
    // If multithreaded compaction is being used *and* a single CompactionFilter
    // instance was supplied via Options::compaction_filter, this method may be
    // called from different threads concurrently.  The application must ensure
    // that the call is thread-safe.
    //
    // If the CompactionFilter was created by a factory, then it will only ever
    // be used by a single thread that is doing the compaction run, and this
    // call does not need to be thread-safe.  However, multiple filters may be
    // in existence and operating concurrently.
    //
    // The last paragraph is not true if you set max_subcompactions to more than
    // 1. In that case, subcompaction from multiple threads may call a single
    // CompactionFilter concurrently.
    //
    // For rust:
    // - None: false, indicates that the kv should be preserved in the output of this compaction run.
    // - Some(None): true, indicates that this key-value should be removed from the output of the
    //   compaction.
    // - Some(Some(vec![])): modify the existing_value and pass it back through new_value.
    // fn filter(&self, level: u32, key: &[u8], existing_value: &[u8]) -> Option<Option<Vec<u8>>> {
    // None
    // }
    //
    // The compaction process invokes this method on every merge operand. If this
    // method returns true, the merge operand will be ignored and not written out
    // in the compaction output
    //
    // Note: If you are using a TransactionDB, it is not recommended to implement
    // FilterMergeOperand().  If a Merge operation is filtered out, TransactionDB
    // may not realize there is a write conflict and may allow a Transaction to
    // Commit that should have failed.  Instead, it is better to implement any
    // Merge filtering inside the MergeOperator.
    // fn filter_merge_operand(&self, level: u32, key: &[u8], operand: &[u8]) -> bool {
    // false
    // }
    //
    /// An extended API. Called for both values and merge operands.
    /// Allows changing value and skipping ranges of keys.
    /// The default implementation uses Filter() and FilterMergeOperand().
    /// If you're overriding this method, no need to override the other two.
    /// `value_type` indicates whether this key-value corresponds to a normal
    /// value (e.g. written with Put())  or a merge operand (written with Merge()).
    ///
    /// Possible return values:
    ///  * kKeep - keep the key-value pair.
    ///  * kRemove - remove the key-value pair or merge operand.
    ///  * kChangeValue - keep the key and change the value/operand to *new_value.
    ///  * kRemoveAndSkipUntil - remove this key-value pair, and also remove all key-value pairs
    ///    with key in [key, *skip_until). This range of keys will be skipped without reading,
    ///    potentially saving some IO operations compared to removing the keys one by one.
    ///
    ///    *skip_until <= key is treated the same as Decision::kKeep
    ///    (since the range [key, *skip_until) is empty).
    ///
    ///    The keys are skipped even if there are snapshots containing them,
    ///    as if IgnoreSnapshots() was true; i.e. values removed
    ///    by kRemoveAndSkipUntil can disappear from a snapshot - beware
    ///    if you're using TransactionDB or DB::GetSnapshot().
    ///
    ///    Another warning: if value for a key was overwritten or merged into
    ///    (multiple Put()s or Merge()s), and compaction filter skips this key
    ///    with kRemoveAndSkipUntil, it's possible that it will remove only
    ///    the new value, exposing the old value that was supposed to be
    ///    overwritten.
    ///
    ///    If you use kRemoveAndSkipUntil, consider also reducing
    ///    compaction_readahead_size option.
    ///
    /// Note: If you are using a TransactionDB, it is not recommended to filter
    /// out or modify merge operands (ValueType::kMergeOperand).
    /// If a merge operation is filtered out, TransactionDB may not realize there
    /// is a write conflict and may allow a Transaction to Commit that should have
    /// failed. Instead, it is better to implement any Merge filtering inside the
    /// MergeOperator.
    ///
    /// Rust:
    ///   Decision for detailed return type.
    fn filter(&mut self, level: i32, key: &[u8], value_type: ValueType, existing_value: &[u8]) -> Decision {
        Decision::Keep
    }

    /// This function is deprecated. Snapshots will always be ignored for
    /// compaction filters, because we realized that not ignoring snapshots doesn't
    /// provide the gurantee we initially thought it would provide. Repeatable
    /// reads will not be guaranteed anyway. If you override the function and
    /// returns false, we will fail the compaction.
    fn ignore_snapshots(&self) -> bool {
        true
    }

    /// Returns a name that identifies this compaction filter.
    /// The name will be printed to LOG file on start up for diagnosis.
    fn name(&self) -> &str {
        "RustCompactionFilterV2\0"
    }
}

/// Each compaction will create a new `CompactionFilter` allowing the
/// application to know about different compactions
pub trait CompactionFilterFactory {
    fn create_compaction_filter(&self, context: &Context) -> Box<dyn CompactionFilter>;

    /// Returns a name that identifies this compaction filter factory.
    fn name(&self) -> &str {
        "RustCompactionFilterFactory\0"
    }
}

/// Context information of a compaction run
#[repr(C)]
pub struct Context {
    /// Does this compaction run include all data files
    pub is_full_compaction: bool,
    /// Is this compaction requested by the client (true),
    /// or is it occurring as an automatic compaction process
    pub is_manual_compaction: bool,
    /// Which column family this compaction is for.
    pub column_family_id: u32,
}

// call rust fn in C
#[doc(hidden)]
pub mod c {
    use super::*;

    #[no_mangle]
    #[allow(mutable_transmutes)]
    pub unsafe extern "C" fn rust_compaction_filter_call(
        f: *mut (),
        level: c_int,
        key: &&[u8], // *Slice
        value_type: ValueType,
        existing_value: &&[u8], // *Slice
        new_value: *mut (),     // *std::string
        skip_until: *mut (),
    ) -> c_int {
        assert!(!f.is_null());
        // FIXME: borrow as mutable
        let filter = f as *mut &mut (dyn CompactionFilter + Sync);
        // must be the same as C part
        match (*filter).filter(level, key, value_type, existing_value) {
            Decision::Keep => 0,
            Decision::Remove => 1,
            Decision::ChangeValue(nval) => {
                ll::cxx_string_assign(new_value as *mut _, nval.as_ptr() as *const _, nval.len());
                2
            },
            Decision::RemoveAndSkipUntil(skip) => {
                ll::cxx_string_assign(skip_until as *mut _, skip.as_ptr() as *const _, skip.len());
                3
            },
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_compaction_filter_drop(f: *mut ()) {
        assert!(!f.is_null());
        let filter = f as *mut &(dyn CompactionFilter + Sync);
        Box::from_raw(filter);
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_compaction_filter_name(f: *mut ()) -> *const c_char {
        assert!(!f.is_null());
        let filter = f as *mut &(dyn CompactionFilter + Sync);
        (*filter).name().as_ptr() as _
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_compaction_filter_ignore_snapshots(f: *mut ()) -> c_char {
        assert!(!f.is_null());
        let filter = f as *mut &(dyn CompactionFilter + Sync);
        (*filter).ignore_snapshots() as _
    }
}

#[cfg(test)]
mod tests {
    use crate::rocksdb::*;
    use super::*;
    use lazy_static::lazy_static;

    pub struct MyCompactionFilter;

    impl CompactionFilter for MyCompactionFilter {
        fn filter(&mut self, level: i32, key: &[u8], value_type: ValueType, existing_value: &[u8]) -> Decision {
            assert_eq!(value_type, ValueType::Value); // haven't set up merge test

            if existing_value == b"TO-BE-DELETED" {
                Decision::Remove
            } else if existing_value == b"an-typo-in-value" {
                Decision::ChangeValue(b"a-typo-not-in-value".to_vec())
            } else if key == b"key-0" {
                Decision::RemoveAndSkipUntil(b"key-5".to_vec())
            } else {
                Decision::Keep
            }
        }
    }

    lazy_static! {
        static ref MY_COMPACTION_FILTER: MyCompactionFilter = MyCompactionFilter;
    }

    #[test]
    fn compaction_filter() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(
            Options::default()
                .map_db_options(|db| db.create_if_missing(true))
                .map_cf_options(|cf| cf.compaction_filter(&*MY_COMPACTION_FILTER)),
            &tmp_dir,
        )
        .unwrap();

        println!("compact and try remove range");
        assert!(db.put(&WriteOptions::default(), b"key-0", b"23333").is_ok());
        assert!(db.put(&WriteOptions::default(), b"key-1", b"23333").is_ok());
        assert!(db.put(&WriteOptions::default(), b"key-2", b"23333").is_ok());
        assert!(db.put(&WriteOptions::default(), b"key-3", b"23333").is_ok());
        assert!(db.put(&WriteOptions::default(), b"key-4", b"23333").is_ok());
        // following will be reserved
        assert!(db.put(&WriteOptions::default(), b"key-5", b"23333").is_ok());
        assert!(db.put(&WriteOptions::default(), b"key-6", b"23333").is_ok());
        assert!(db.put(&WriteOptions::default(), b"key-7", b"23333").is_ok());
        assert!(db.put(&WriteOptions::default(), b"key-8", b"23333").is_ok());

        println!("compact and delete");
        assert!(db
            .put(&WriteOptions::default(), b"will-delete-me", b"TO-BE-DELETED")
            .is_ok());

        println!("compact and change value");
        assert!(db
            .put(&WriteOptions::default(), b"will-fix-me", b"an-typo-in-value")
            .is_ok());

        // now compact full range
        let ret = db.compact_range(&Default::default(), ..);
        assert!(ret.is_ok(), "error: {:?}", ret);

        assert!(db.get(&ReadOptions::default(), b"will-delete-me").is_err());
        assert!(db
            .get(&ReadOptions::default(), b"will-delete-me")
            .unwrap_err()
            .is_not_found());

        assert!(db.get(&ReadOptions::default(), b"key-0").is_err());
        assert!(db.get(&ReadOptions::default(), b"key-0").unwrap_err().is_not_found());

        assert!(db.get(&ReadOptions::default(), b"key-4").is_err());
        assert!(db.get(&ReadOptions::default(), b"key-4").unwrap_err().is_not_found());

        assert_eq!(db.get(&ReadOptions::default(), b"key-5").unwrap(), b"23333");

        assert_eq!(
            db.get(&ReadOptions::default(), b"will-fix-me").unwrap(),
            b"a-typo-not-in-value"
        );

        drop(db);
        drop(tmp_dir);
    }
}
