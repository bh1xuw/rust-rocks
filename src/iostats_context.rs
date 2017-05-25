//! A thread local context for gathering io-stats efficiently and transparently.
//!
//! Use `SetPerfLevel(PerfLevel::kEnableTime)` to enable time stats.


use std::fmt;

use rocks_sys as ll;

/// A thread local context for gathering io-stats efficiently and transparently.
#[derive(Debug)]
#[repr(C)]
pub struct IOStatsContext {
    /// the thread pool id
    pub thread_pool_id: u64,

    /// number of bytes that has been written.
    pub bytes_written: u64,
    /// number of bytes that has been read.
    pub bytes_read: u64,

    /// time spent in `open()` and `fopen()`.
    pub open_nanos: u64,
    /// time spent in `fallocate()`.
    pub allocate_nanos: u64,
    /// time spent in `write()` and `pwrite()`.
    pub write_nanos: u64,
    /// time spent in `read()` and `pread()`
    pub read_nanos: u64,
    /// time spent in `sync_file_range()`.
    pub range_sync_nanos: u64,
    /// time spent in fsync
    pub fsync_nanos: u64,
    /// time spent in preparing write (fallocate etc).
    pub prepare_write_nanos: u64,
    /// time spent in `Logger::Logv()`.
    pub logger_nanos: u64,
}

impl IOStatsContext {
    /// IOStatsContext for current thread
    pub fn current() -> &'static mut IOStatsContext {
        unsafe {
            let ptr = ll::rocks_get_iostats_context() as *mut IOStatsContext;
            ptr.as_mut().unwrap()
        }
    }

    /// reset all io-stats counter to zero
    pub fn reset(&mut self) {
        unsafe {
            let ptr = self as *mut IOStatsContext as *mut ll::rocks_iostats_context_t;
            ll::rocks_iostats_context_reset(ptr);
        }
    }
}

impl fmt::Display for IOStatsContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        let ptr = self as *const IOStatsContext as *const ll::rocks_iostats_context_t;
        let exclude_zero_counters = false;
        unsafe {
            ll::rocks_iostats_context_to_string(ptr,
                                                exclude_zero_counters as u8,
                                                &mut s as *mut String as *mut _);
        }
        write!(f, "{}", s)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::rocksdb::*;

    #[test]
    fn iostats_context() {
        set_perf_level(PerfLevel::EnableTime);

        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(Options::default()
                          .map_db_options(|db| db.create_if_missing(true)),
                          &tmp_dir)
            .unwrap();

        assert!(db.put(&Default::default(), b"long-key", vec![b'A'; 1024 * 1024].as_ref())
                .is_ok());
        assert!(db.put(&Default::default(), b"a", b"1").is_ok());
        assert!(db.put(&Default::default(), b"b", b"2").is_ok());
        assert!(db.put(&Default::default(), b"c", b"3").is_ok());

        assert!(db.compact_range(&Default::default(), ..).is_ok());

        let stat = IOStatsContext::current();

        assert!(stat.bytes_written > 1024);

        println!("dbg => {:?}", stat);
        println!("show => {}", stat);

        stat.reset();
        assert_eq!(stat.bytes_written, 0);

        println!("dbg => {:?}", stat);
        println!("show => {}", stat);

        // FIXME: why thread_pool changes?
    }
}
