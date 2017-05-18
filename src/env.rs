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
    raw: *mut ll::rocks_envoptions_t,
}

impl Drop for EnvOptions {
    fn drop(&mut self) {
        unsafe { ll::rocks_envoptions_destroy(self.raw) }
    }
}

impl EnvOptions {
    pub fn raw(&self) -> *mut ll::rocks_envoptions_t {
        self.raw
    }

    /// If true, then use mmap to read data
    pub fn use_mmap_reads(self, val: bool) -> Self {
        unsafe {
            ll::rocks_envoptions_set_use_mmap_reads(self.raw, val as u8);
        }
        self
    }

    /// If true, then use mmap to write data
    pub fn use_mmap_writes(self, val: bool) -> Self {
        unsafe {
            ll::rocks_envoptions_set_use_mmap_writes(self.raw, val as u8);
        }
        self
    }

    /// If true, then use O_DIRECT for reading data
    pub fn use_direct_reads(self, val: bool) -> Self {
        unsafe {
            ll::rocks_envoptions_set_use_direct_reads(self.raw, val as u8);
        }
        self
    }

    /// If true, then use O_DIRECT for writing data
    pub fn use_direct_writes(self, val: bool) -> Self {
        unsafe {
            ll::rocks_envoptions_set_use_direct_writes(self.raw, val as u8);
        }
        self
    }

    /// If false, fallocate() calls are bypassed
    pub fn allow_fallocate(self, val: bool) -> Self {
        unsafe {
            ll::rocks_envoptions_set_allow_fallocate(self.raw, val as u8);
        }
        self
    }

    /// If true, set the FD_CLOEXEC on open fd.
    pub fn fd_cloexec(self, val: bool) -> Self {
        unsafe {
            ll::rocks_envoptions_set_fd_cloexec(self.raw, val as u8);
        }
        self
    }

    /// Allows OS to incrementally sync files to disk while they are being
    /// written, in the background. Issue one request for every bytes_per_sync
    /// written. 0 turns it off.
    /// Default: 0
    pub fn bytes_per_sync(self, val: u64) -> Self {
        unsafe {
            ll::rocks_envoptions_set_bytes_per_sync(self.raw, val);
        }
        self
    }

    /// If true, we will preallocate the file with FALLOC_FL_KEEP_SIZE flag, which
    /// means that file size won't change as part of preallocation.
    /// If false, preallocation will also change the file size. This option will
    /// improve the performance in workloads where you sync the data on every
    /// write. By default, we set it to true for MANIFEST writes and false for
    /// WAL writes
    pub fn fallocate_with_keep_size(self, val: bool) -> Self {
        unsafe {
            ll::rocks_envoptions_set_fallocate_with_keep_size(self.raw, val as u8);
        }
        self
    }

    /// See DBOPtions doc
    pub fn compaction_readahead_size(self, val: usize) -> Self {
        unsafe {
            ll::rocks_envoptions_set_compaction_readahead_size(self.raw, val);
        }
        self
    }

    /// See DBOPtions doc
    pub fn random_access_max_buffer_size(self, val: usize) -> Self {
        unsafe {
            ll::rocks_envoptions_set_random_access_max_buffer_size(self.raw, val);
        }
        self
    }

    /// See DBOptions doc
    pub fn writable_file_max_buffer_size(self, val: usize) -> Self {
        unsafe {
            ll::rocks_envoptions_set_writable_file_max_buffer_size(self.raw, val);
        }
        self
    }

    // If not nullptr, write rate limiting is enabled for flush and compaction
    /*
    pub fn rate_limiter(self, val: Option<RateLimiter>) -> Self {
    unsafe {
    ll::rocks_envoptions_set_
    }
    self
     */
}

impl Default for EnvOptions {
    fn default() -> Self {
        EnvOptions { raw: unsafe { ll::rocks_envoptions_create() } }
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


/// An interface for writing log messages.
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
