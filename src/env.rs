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
use std::str;
use std::path::Path;
use std::ffi::CStr;
use lazy_static::lazy_static;

use rocks_sys as ll;

use crate::error::Status;
use crate::to_raw::{ToRaw, FromRaw};
use crate::thread_status::ThreadStatus;
use crate::Result;

pub const DEFAULT_PAGE_SIZE: usize = 4 * 1024;

lazy_static! {
    static ref DEFAULT_ENVOPTIONS: EnvOptions = {
        EnvOptions::default()
    };

    static ref DEFAULT_ENV: Env = {
        Env { raw: unsafe { ll::rocks_create_default_env() } }
    };
}

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

impl Default for EnvOptions {
    fn default() -> Self {
        EnvOptions { raw: unsafe { ll::rocks_envoptions_create() } }
    }
}

unsafe impl Sync for EnvOptions {}

impl EnvOptions {
    pub fn default_instance() -> &'static EnvOptions {
        &*DEFAULT_ENVOPTIONS
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
    //
    // pub fn rate_limiter(self, val: Option<RateLimiter>) -> Self {
    // unsafe {
    // ll::rocks_envoptions_set_
    // }
    // self
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
            ll::rocks_logger_log(self.raw, mem::transmute(log_level), msg.as_ptr() as *const _, msg.len());
        }
    }

    /// Flush to the OS buffers
    pub fn flush(&self) {
        unsafe {
            ll::rocks_logger_flush(self.raw);
        }
    }

    pub fn get_log_level(&self) -> InfoLogLevel {
        unsafe { mem::transmute(ll::rocks_logger_get_log_level(self.raw)) }
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

unsafe impl Sync for Env {}

impl Env {
    /// Return a default environment suitable for the current operating
    /// system.  Sophisticated users may wish to provide their own Env
    /// implementation instead of relying on this default environment.
    ///
    /// The result of Default() belongs to rocksdb and must never be deleted.
    pub fn default_instance() -> &'static Env {
        &*DEFAULT_ENV
    }

    /// Returns a new environment that stores its data in memory and delegates
    /// all non-file-storage tasks to base_env.
    ///
    /// FIXME: missing base_env
    pub fn new_mem() -> Env {
        Env { raw: unsafe { ll::rocks_create_mem_env() } }
    }

    /// Returns a new environment that measures function call times for filesystem
    /// operations, reporting results to variables in PerfContext.
    ///
    /// This is a factory method for TimedEnv defined in utilities/env_timed.cc.
    ///
    /// FIXME: missing base_env
    pub fn new_timed() -> Env {
        Env { raw: unsafe { ll::rocks_create_timed_env() } }
    }


    /// The number of background worker threads of a specific thread pool
    pub fn set_low_priority_background_threads(&self, number: i32) {
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
        unsafe { ll::rocks_env_get_thread_pool_queue_len(self.raw, mem::transmute(pri)) as u32 }
    }

    /// Create and return a log file for storing informational messages.
    pub fn create_logger<P: AsRef<Path>>(&self, fname: P) -> Result<Logger> {
        let mut status = ptr::null_mut();
        unsafe {
            let name = fname.as_ref().to_str().unwrap();
            let logger = ll::rocks_env_new_logger(self.raw, name.as_ptr() as *const _, name.len(), &mut status);
            Status::from_ll(status).map(|_| Logger::from_ll(logger))
        }
    }

    /// Returns the number of micro-seconds since some fixed point in time.
    /// It is often used as system time such as in GenericRateLimiter
    /// and other places so a port needs to return system time in order to work.
    pub fn now_micros(&self) -> u64 {
        unsafe { ll::rocks_env_now_micros(self.raw) as u64 }
    }

    /// Returns the number of nano-seconds since some fixed point in time. Only
    /// useful for computing deltas of time in one run.
    /// Default implementation simply relies on NowMicros.
    /// In platform-specific implementations, NowNanos() should return time points
    /// that are MONOTONIC.
    pub fn now_nanos(&self) -> u64 {
        unsafe { ll::rocks_env_now_nanos(self.raw) as u64 }
    }

    /// Sleep/delay the thread for the perscribed number of micro-seconds.
    pub fn sleep_for_microseconds(&self, micros: i32) {
        unsafe {
            ll::rocks_env_sleep_for_microseconds(self.raw, micros);
        }
    }

    /// Get the current host name.
    pub fn get_hostname(&self) -> Result<String> {
        let mut buf = [0u8; 128];
        let mut status = ptr::null_mut();
        unsafe {
            ll::rocks_env_get_host_name(self.raw, (&mut buf).as_mut_ptr() as *mut _, 128, &mut status);
            Status::from_ll(status)
                .map(|()| CStr::from_ptr((&buf).as_ptr() as *const _))
                .and_then(|s| s.to_str().map_err(|_| Status::with_message("utf8 error")))
                .map(|s| s.into())
        }
    }

    /// Get the number of seconds since the Epoch, 1970-01-01 00:00:00 (UTC).
    /// Only overwrites *unix_time on success.
    pub fn get_current_time(&self) -> Result<u64> {
        let mut status = ptr::null_mut();
        unsafe {
            let tm = ll::rocks_env_get_current_time(self.raw, &mut status);
            Status::from_ll(status)
                .map(|()| tm as u64)
        }
    }

    /// Converts seconds-since-Jan-01-1970 to a printable string
    pub fn time_to_string(&self, time: u64) -> String {
        unsafe {
            let cxx_string = ll::rocks_env_time_to_string(self.raw, time);
            let ret = CStr::from_ptr(ll::cxx_string_data(cxx_string) as *const _)
                .to_str()
                .unwrap()
                .into();
            ll::cxx_string_destroy(cxx_string);
            ret
        }
    }

    /// The number of background worker threads of a specific thread pool
    /// for this environment. 'LOW' is the default pool.
    ///
    /// default number: 1
    ///
    /// FIXME: &mut self ?
    pub fn set_background_threads(&self, number: i32, pri: Priority) {
        match pri {
            Priority::Low => self.set_low_priority_background_threads(number),
            Priority::High => self.set_high_priority_background_threads(number),
            _ => unreachable!("wrong pri for thread pool"),
        }
    }

    pub fn get_background_threads(&self, pri: Priority) -> i32 {
        unsafe { ll::rocks_env_get_background_threads(self.raw, mem::transmute(pri)) as i32 }
    }

    /// Enlarge number of background worker threads of a specific thread pool
    /// for this environment if it is smaller than specified. 'LOW' is the default
    /// pool.
    pub fn inc_background_threads_if_needed(&self, number: i32, pri: Priority) {
        unsafe {
            ll::rocks_env_inc_background_threads_if_needed(self.raw, number, mem::transmute(pri));
        }
    }

    /// Lower IO priority for threads from the specified pool.
    pub fn lower_thread_pool_io_priority(&self, pool: Priority) {
        unsafe {
            ll::rocks_env_lower_thread_pool_io_priority(self.raw, mem::transmute(pool));
        }
    }

    /// Returns the status of all threads that belong to the current Env.
    pub fn get_thread_list(&self) -> Vec<ThreadStatus> {
        let mut len = 0;
        unsafe {
            let thread_status_arr = ll::rocks_env_get_thread_list(self.raw, &mut len);
            let ret = (0..len)
                .into_iter()
                .map(|i| ThreadStatus::from_ll(*thread_status_arr.offset(i as isize)))
                .collect();
            ll::rocks_env_get_thread_list_destroy(thread_status_arr);
            ret
        }
    }

    /// Returns the ID of the current thread.
    pub fn get_thread_id(&self) -> u64 {
        unsafe { ll::rocks_env_get_thread_id(self.raw) as u64 }
    }
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::prelude::*;
    use super::*;

    #[test]
    fn env_basic() {
        let env = Env::default_instance();

        assert!(env.get_thread_id() > 0);
        assert!(env.now_micros() > 1500000000000000);
        assert!(env.get_hostname().is_ok());
        assert!(env.get_current_time().is_ok());
        assert!(env.time_to_string(env.get_current_time().unwrap()).len() > 10);
    }

    #[test]
    fn logger() {
        let log_dir = ::tempdir::TempDir::new_in(".", "log").unwrap();
        let env = Env::default_instance();

        {
            let logger = env.create_logger(log_dir.path().join("test.log"));
            assert!(logger.is_ok());

            let mut logger = logger.unwrap();

            logger.set_log_level(InfoLogLevel::Info);
            assert_eq!(logger.get_log_level(), InfoLogLevel::Info);

            logger.log(InfoLogLevel::Error, "test log message");
            logger.log(InfoLogLevel::Debug, "debug log message");
            logger.flush();
        }

        let mut f = File::open(log_dir.path().join("test.log")).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();

        assert!(s.contains("[ERROR] test log message"));
        assert!(!s.contains("debug log message"));
    }
}
