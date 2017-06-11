//! An Env is an interface used by the rocksdb implementation to access
//! operating system functionality like the filesystem etc.
//!
//! Callers may wish to provide a custom Env object when opening a database to
//! get fine gain control; e.g., to rate limit file system operations.
//!
//! All Env implementations are safe for concurrent access from
//! multiple threads without any external synchronization.

use std::mem;
use std::ptr;
use std::path::Path;

use rocks_sys as ll;

use error::Status;
use super::Result;
use rate_limiter::RateLimiter;
use to_raw::ToRaw;


pub const DEFAULT_PAGE_SIZE: usize = 4 * 1024;

/// Priority for scheduling job in thread pool
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Priority {
    Low,
    High,
    Total,
}

/// Options while opening a file to read/write
pub struct EnvOptions {
    raw: *mut ll::rocks_envoptions_t,
}

impl Drop for EnvOptions {
    fn drop(&mut self) {
        unsafe { ll::rocks_envoptions_destroy(self.raw) }
    }
}

impl ToRaw<ll::rocks_envoptions_t> for EnvOptions {
    fn raw(&self) -> *mut ll::rocks_envoptions_t {
        self.raw
    }
}

impl EnvOptions {
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

/// Log levels for `Logger`
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum InfoLogLevel {
    Debug = 0,
    Info,
    Warn,
    Error,
    Fatal,
    Header,
}


/// An interface for writing log messages.
#[derive(Debug)]
pub struct Logger {
    raw: *mut ll::rocks_logger_t,
}

impl ToRaw<ll::rocks_logger_t> for Logger {
    fn raw(&self) -> *mut ll::rocks_logger_t {
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

impl Logger {
    unsafe fn from_ll(raw: *mut ll::rocks_logger_t) -> Logger {
        Logger { raw: raw }
    }

    /// Write an entry to the log file with the specified log level
    /// and format.  Any log with level under the internal log level
    /// of *this (see @SetInfoLogLevel and @GetInfoLogLevel) will not be
    /// printed.
    pub fn log(&self, log_level: InfoLogLevel, msg: &str) {
        unsafe {
            ll::rocks_logger_log(self.raw,
                                 mem::transmute(log_level),
                                 msg.as_ptr() as *const _,
                                 msg.len());
        }
    }

    /// Flush to the OS buffers
    pub fn flush(&self) {
        unsafe {
            ll::rocks_logger_flush(self.raw);
        }
    }

    pub fn get_log_level(&self) -> InfoLogLevel {
        unsafe {
            mem::transmute(ll::rocks_logger_get_log_level(self.raw))
        }
    }

    pub fn set_log_level(&mut self, log_level: InfoLogLevel) {
        unsafe {
            ll::rocks_logger_set_log_level(self.raw, mem::transmute(log_level));
        }
    }
}


/// An `Env` is an interface used by the rocksdb implementation to access
/// operating system functionality like the filesystem etc.
pub struct Env {
    raw: *mut ll::rocks_env_t,
}

impl ToRaw<ll::rocks_env_t> for Env {
    fn raw(&self) -> *mut ll::rocks_env_t {
        self.raw
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        unsafe {
            // ffi function will skip dealloc Env::Default() ptr
            ll::rocks_env_destroy(self.raw)
        }
    }
}

impl Default for Env {
    /// Return a default environment suitable for the current operating
    /// system.  Sophisticated users may wish to provide their own Env
    /// implementation instead of relying on this default environment.
    ///
    /// The result of Default() belongs to rocksdb and must never be deleted.
    fn default() -> Self {
        Env {
            raw: unsafe { ll::rocks_create_default_env() },
        }
    }
}

impl Env {
    /// Returns a new environment that stores its data in memory and delegates
    /// all non-file-storage tasks to base_env.
    pub fn new_mem() -> Env {
        Env {
            raw: unsafe { ll::rocks_create_mem_env() },
        }
    }

    /// The number of background worker threads of a specific thread pool
    pub fn set_background_threads(&self, number: i32) {
        unsafe {
            ll::rocks_env_set_background_threads(self.raw, number);
        }
    }

    /// The number of background worker threads of a high priority thread pool
    pub fn set_high_priority_background_threads(&self, number: i32) {
        unsafe {
            ll::rocks_env_set_high_priority_background_threads(self.raw, number);
        }
    }

    /// Wait for all threads started by StartThread to terminate.
    pub fn wait_for_join(&self) {
        unsafe {
            ll::rocks_env_join_all_threads(self.raw);
        }
    }

    /// Get thread pool queue length for specific thrad pool.
    pub fn get_thread_pool_queue_len(&self, pri: Priority) -> u32 {
        unsafe {
            ll::rocks_env_get_thread_pool_queue_len(self.raw, mem::transmute(pri)) as u32
        }
    }

    /// Create and return a log file for storing informational messages.
    pub fn create_logger<P: AsRef<Path>>(&self, fname: P) -> Result<Logger> {
        unsafe {
            let mut status = ptr::null_mut();
            let name = fname.as_ref().to_str().unwrap();
            let logger = ll::rocks_env_new_logger(self.raw,
                                                  name.as_ptr() as *const _,
                                                  name.len(),
                                                  &mut status);
            Status::from_ll(status).map(|_| Logger::from_ll(logger))
        }
    }
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::prelude::*;
    use super::*;

    #[test]
    fn logger() {
        let log_dir = ::tempdir::TempDir::new_in(".", "log").unwrap();
        let env = Env::default();

        {
            let logger = env.create_logger(log_dir.path().join("./test.log"));
            assert!(logger.is_ok());

            let mut logger = logger.unwrap();

            logger.set_log_level(InfoLogLevel::Info);
            assert_eq!(logger.get_log_level(), InfoLogLevel::Info);

            logger.log(InfoLogLevel::Error, "test log message");

            logger.log(InfoLogLevel::Debug, "debug log message");

            logger.flush();

        }

        let mut f = File::open(log_dir.path().join("./test.log")).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();

        assert!(s.contains("[ERROR] test log message"));
        assert!(!s.contains("debug log message"));
    }
}

