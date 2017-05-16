//! An Env is an interface used by the rocksdb implementation to access
//! operating system functionality like the filesystem etc.  Callers
//! may wish to provide a custom Env object when opening a database to
//! get fine gain control; e.g., to rate limit file system operations.
//!
//! All Env implementations are safe for concurrent access from
//! multiple threads without any external synchronization.


use rocks_sys as ll;

use rate_limiter::RateLimiter;


pub const DEFAULT_PAGE_SIZE: usize = 4 * 1024;

/// Options while opening a file to read/write
pub struct EnvOptions {
    /// If true, then use mmap to read data
    pub use_mmap_reads: bool,

    /// If true, then use mmap to write data
    pub use_mmap_writes: bool,

    /// If true, then use O_DIRECT for reading data
    pub use_direct_reads: bool,

    /// If true, then use O_DIRECT for writing data
    pub use_direct_writes: bool,

    /// If false, fallocate() calls are bypassed
    pub allow_fallocate: bool,

    /// If true, set the FD_CLOEXEC on open fd.
    pub set_fd_cloexec: bool,

    /// Allows OS to incrementally sync files to disk while they are being
    /// written, in the background. Issue one request for every bytes_per_sync
    /// written. 0 turns it off.
    /// Default: 0
    pub bytes_per_sync: u64,

    /// If true, we will preallocate the file with FALLOC_FL_KEEP_SIZE flag, which
    /// means that file size won't change as part of preallocation.
    /// If false, preallocation will also change the file size. This option will
    /// improve the performance in workloads where you sync the data on every
    /// write. By default, we set it to true for MANIFEST writes and false for
    /// WAL writes
    pub fallocate_with_keep_size: bool,

    /// See DBOPtions doc
    pub compaction_readahead_size: usize,

    /// See DBOPtions doc
    pub random_access_max_buffer_size: usize,

    /// See DBOptions doc
    pub writable_file_max_buffer_size: usize,

    /// If not nullptr, write rate limiting is enabled for flush and compaction
    pub rate_limiter: Option<RateLimiter>,
}


impl Default for EnvOptions {
    fn default() -> Self {
        EnvOptions {
            use_mmap_reads: false,
            use_mmap_writes: true,
            use_direct_reads: false,
            use_direct_writes: false,
            allow_fallocate: true,
            set_fd_cloexec: true,
            bytes_per_sync: 0,
            fallocate_with_keep_size: true,
            compaction_readahead_size: 0,
            random_access_max_buffer_size: 0,
            writable_file_max_buffer_size: 1024 * 1024,
            rate_limiter: None,
        }
    }
}



#[repr(C)]
pub enum InfoLogLevel {
    Debug = 0,
    Info,
    Warn,
    Error,
    Fatal,
    Header,
}


pub struct Logger {
    raw: *mut ll::rocks_logger_t,
}

impl Logger {
    pub unsafe fn from_ll(raw: *mut ll::rocks_logger_t) -> Logger {
        Logger { raw: raw }
    }

    pub fn raw(&self) -> *mut ll::rocks_logger_t {
        self.raw
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_logger_destroy(self.raw);
        }
    }
}

pub struct Env;
