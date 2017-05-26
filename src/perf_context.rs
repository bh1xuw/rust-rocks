//! A thread local context for gathering performance counter efficiently
//! and transparently.

use std::fmt;

use rocks_sys as ll;

/// A thread local context for gathering performance counter efficiently
/// and transparently.
///
/// Use `SetPerfLevel(PerfLevel::kEnableTime)` to enable time stats.
#[derive(Debug)]
#[repr(C)]
pub struct PerfContext {
    /// total number of user key comparisons
    pub user_key_comparison_count: u64,
    /// total number of block cache hits
    pub block_cache_hit_count: u64,
    /// total number of block reads (with IO)
    pub block_read_count: u64,
    /// total number of bytes from block reads
    pub block_read_byte: u64,
    /// total nanos spent on block reads
    pub block_read_time: u64,
    /// total nanos spent on block checksum
    pub block_checksum_time: u64,
    /// total nanos spent on block decompression
    pub block_decompress_time: u64,
    /// total number of internal keys skipped over during iteration.
    ///
    /// There are several reasons for it:
    /// 1. when calling `Next()`, the iterator is in the position of the previous
    ///    key, so that we'll need to skip it. It means this counter will always
    ///    be incremented in `Next()`.
    /// 2. when calling `Next()`, we need to skip internal entries for the previous
    ///    keys that are overwritten.
    /// 3. when calling `Next()`, `Seek()` or `SeekToFirst()`, after previous key
    ///    before calling `Next()`, the seek key in `Seek()` or the beginning for
    ///    `SeekToFirst()`, there may be one or more deleted keys before the next
    ///    valid key that the operation should place the iterator to. We need
    ///    to skip both of the tombstone and updates hidden by the tombstones. The
    ///    tombstones are not included in this counter, while previous updates
    ///    hidden by the tombstones will be included here.
    /// 4. symmetric cases for `Prev()` and `SeekToLast()`
    ///
    /// `internal_recent_skipped_count` is not included in this counter.
    pub internal_key_skipped_count: u64,
    /// Total number of deletes and single deletes skipped over during iteration
    ///
    /// When calling `Next()`, `Seek()` or `SeekToFirst()`, after previous position
    /// before calling `Next()`, the seek key in `Seek()` or the beginning for
    /// `SeekToFirst()`, there may be one or more deleted keys before the next valid
    /// key. Every deleted key is counted once. We don't recount here if there are
    /// still older updates invalidated by the tombstones.
    pub internal_delete_skipped_count: u64,
    /// How many times iterators skipped over internal keys that are more recent
    /// than the snapshot that iterator is using.
    pub internal_recent_skipped_count: u64,
    /// How many values were fed into merge operator by iterators.
    pub internal_merge_count: u64,

    /// total nanos spent on getting snapshot
    pub get_snapshot_time: u64,
    /// total nanos spent on querying memtables
    pub get_from_memtable_time: u64,
    /// number of mem tables queried
    pub get_from_memtable_count: u64,
    /// total nanos spent after `Get()` finds a key
    pub get_post_process_time: u64,
    /// total nanos reading from output files
    pub get_from_output_files_time: u64,
    /// total nanos spent on seeking memtable
    pub seek_on_memtable_time: u64,
    /// number of seeks issued on memtable
    /// (including `SeekForPrev` but not `SeekToFirst` and `SeekToLast`)
    pub seek_on_memtable_count: u64,
    /// number of `Next()`s issued on memtable
    pub next_on_memtable_count: u64,
    /// number of `Prev()`s issued on memtable
    pub prev_on_memtable_count: u64,
    /// total nanos spent on seeking child iters
    pub seek_child_seek_time: u64,
    /// number of seek issued in child iterators
    pub seek_child_seek_count: u64,
    /// total nanos spent on the merge min heap
    pub seek_min_heap_time: u64,
    /// total nanos spent on the merge max heap
    pub seek_max_heap_time: u64,
    /// total nanos spent on seeking the internal entries
    pub seek_internal_seek_time: u64,
    /// total nanos spent on iterating internal entries to find the next user entry
    pub find_next_user_entry_time: u64,

    /// total nanos spent on writing to WAL
    pub write_wal_time: u64,
    /// total nanos spent on writing to mem tables
    pub write_memtable_time: u64,
    /// total nanos spent on delaying write
    pub write_delay_time: u64,
    /// total nanos spent on writing a record, excluding the above three times
    pub write_pre_and_post_process_time: u64,

    /// time spent on acquiring DB mutex.
    pub db_mutex_lock_nanos: u64,
    /// Time spent on waiting with a condition variable created with DB mutex.
    pub db_condition_wait_nanos: u64,
    /// Time spent on merge operator.
    pub merge_operator_time_nanos: u64,

    /// Time spent on reading index block from block cache or SST file
    pub read_index_block_nanos: u64,
    /// Time spent on reading filter block from block cache or SST file
    pub read_filter_block_nanos: u64,
    /// Time spent on creating data block iterator
    pub new_table_block_iter_nanos: u64,
    /// Time spent on creating a iterator of an SST file.
    pub new_table_iterator_nanos: u64,
    /// Time spent on seeking a key in data/index blocks
    pub block_seek_nanos: u64,
    /// Time spent on finding or creating a table reader
    pub find_table_nanos: u64,
    /// total number of mem table bloom hits
    pub bloom_memtable_hit_count: u64,
    /// total number of mem table bloom misses
    pub bloom_memtable_miss_count: u64,
    /// total number of SST table bloom hits
    pub bloom_sst_hit_count: u64,
    /// total number of SST table bloom misses
    pub bloom_sst_miss_count: u64,
}

impl PerfContext {
    /// PerfContext for current thread
    pub fn current() -> &'static mut PerfContext {
        unsafe {
            let ptr = ll::rocks_get_perf_context() as *mut PerfContext;
            ptr.as_mut().unwrap()
        }
    }

    /// reset all performance counters to zero
    pub fn reset(&mut self) {
        unsafe {
            let ptr = self as *mut PerfContext as *mut ll::rocks_perf_context_t;
            ll::rocks_perf_context_reset(ptr);
        }
    }
}

impl fmt::Display for PerfContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        let ptr = self as *const PerfContext as *const ll::rocks_perf_context_t;
        let exclude_zero_counters = false;
        unsafe {
            ll::rocks_perf_context_to_string(ptr, exclude_zero_counters as u8, &mut s as *mut String as *mut _);
        }
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::rocksdb::*;

    #[test]
    fn perf_context() {
        set_perf_level(PerfLevel::EnableTime);

        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(Options::default().map_db_options(|db| db.create_if_missing(true)), &tmp_dir).unwrap();

        assert!(db.put(&Default::default(), b"long-key", vec![b'A'; 1024 * 1024].as_ref())
                .is_ok());
        assert!(db.put(&Default::default(), b"a", b"1").is_ok());
        assert!(db.put(&Default::default(), b"b", b"2").is_ok());
        assert!(db.put(&Default::default(), b"c", b"3").is_ok());

        assert!(db.compact_range(&Default::default(), ..).is_ok());

        assert!(db.get(&Default::default(), b"long-key").is_ok());

        let stat = PerfContext::current();

        assert!(stat.block_read_count > 0);
        assert!(stat.user_key_comparison_count > 0);

        assert!(stat.to_string().len() > 200);

        println!("dbg => {:?}", stat);
        println!("show => {}", stat);

        stat.reset();
        assert_eq!(stat.user_key_comparison_count, 0);
    }
}
