//! This file defines the structures for exposing run-time status of any
//! rocksdb-related thread.  Such run-time status can be obtained via
//! GetThreadList() API.
//!
//! Note that all thread-status features are still under-development, and
//! thus APIs and class definitions might subject to change at this point.
//! Will remove this comment once the APIs have been finalized.

use std::mem;
use std::str;
use std::slice;
use std::fmt;

use rocks_sys as ll;

use crate::to_raw::FromRaw;

/// The type of a thread.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum ThreadType {
    /// RocksDB BG thread in high-pri thread pool
    HighPriority = 0,
    /// RocksDB BG thread in low-pri thread pool
    LowPriority,
    /// User thread (Non-RocksDB BG thread)
    User,
}

/// The type used to refer to a thread operation.
/// A thread operation describes high-level action of a thread.
/// Examples include compaction and flush.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum OperationType {
    Unknown = 0,
    Compaction,
    Flush,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum OperationStage {
    Unknown = 0,
    FlushRun,
    FlushWriteL0,
    CompactionPrepare,
    CompactionRun,
    CompactionProcessKv,
    CompactionInstall,
    CompactionSyncFile,
    PickMemtablesToFlush,
    MemtableRollback,
    MemtableInstallFlushResults,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum CompactionPropertyType {
    JobId = 0,
    InputOutputLevel,
    PropFlags,
    TotalInputBytes,
    BytesRead,
    BytesWritten,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum FlushPropertyType {
    JobId = 0,
    BytesMemtables,
    BytesWritten,
}

/// The type used to refer to a thread state.
/// A state describes lower-level action of a thread
/// such as reading / writing a file or waiting for a mutex.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum StateType {
    Unknown = 0,
    MutexWait = 1,
}

/// A structure that describes the current status of a thread.
/// The status of active threads can be fetched using
/// rocksdb::GetThreadList().
#[repr(C)]
pub struct ThreadStatus {
    raw: *mut ll::rocks_thread_status_t,
}

impl FromRaw<ll::rocks_thread_status_t> for ThreadStatus {
    unsafe fn from_ll(raw: *mut ll::rocks_thread_status_t) -> ThreadStatus {
        ThreadStatus { raw: raw }
    }
}

impl Drop for ThreadStatus {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_thread_status_destroy(self.raw);
        }
    }
}

impl fmt::Debug for ThreadStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ThreadStatus")
            .field("id", &self.thread_id())
            .field("type", &self.thread_type())
            .field("db", &self.db_name())
            .field("cf", &self.cf_name())
            .field("operation_type", &self.operation_type())
            .field("op_elapsed_micros", &self.op_elapsed_micros())
            .field("operation_stage", &self.operation_stage())
            .field("state_type", &self.state_type())
            .finish()
    }
}


impl ThreadStatus {
    /// An unique ID for the thread.
    pub fn thread_id(&self) -> u64 {
        unsafe { ll::rocks_thread_status_get_thread_id(self.raw) }
    }

    /// The type of the thread, it could be HIGH_PRIORITY,
    /// LOW_PRIORITY, and USER
    pub fn thread_type(&self) -> ThreadType {
        unsafe { mem::transmute(ll::rocks_thread_status_get_thread_type(self.raw)) }
    }

    /// The name of the DB instance where the thread is currently
    /// involved with.  It would be set to empty string if the thread
    /// does not involve in any DB operation.
    pub fn db_name(&self) -> &str {
        let mut len = 0;
        unsafe {
            let ptr = ll::rocks_thread_status_get_db_name(self.raw, &mut len);
            str::from_utf8_unchecked(slice::from_raw_parts(ptr as *const u8, len))
        }
    }

    /// The name of the column family where the thread is currently
    /// It would be set to empty string if the thread does not involve
    /// in any column family.
    pub fn cf_name(&self) -> &str {
        let mut len = 0;
        unsafe {
            let ptr = ll::rocks_thread_status_get_cf_name(self.raw, &mut len);
            str::from_utf8_unchecked(slice::from_raw_parts(ptr as *const u8, len))
        }
    }

    /// The operation (high-level action) that the current thread is involved.
    pub fn operation_type(&self) -> OperationType {
        unsafe { mem::transmute(ll::rocks_thread_status_get_operation_type(self.raw)) }
    }

    /// The elapsed time of the current thread operation in microseconds.
    pub fn op_elapsed_micros(&self) -> u64 {
        unsafe { ll::rocks_thread_status_get_op_elapsed_micros(self.raw) }
    }

    /// An integer showing the current stage where the thread is involved
    /// in the current operation.
    pub fn operation_stage(&self) -> OperationStage {
        unsafe { mem::transmute(ll::rocks_thread_status_get_operation_stage(self.raw)) }
    }

    /// A list of properties that describe some details about the current
    /// operation.  Same field in op_properties[] might have different
    /// meanings for different operations.
    pub fn op_properties(&self) -> &[u64] {
        let mut len = 0;
        unsafe {
            let ptr = ll::rocks_thread_status_get_op_properties(self.raw, &mut len);
            slice::from_raw_parts(ptr, len)
        }
    }

    /// The state (lower-level action) that the current thread is involved.
    pub fn state_type(&self) -> StateType {
        unsafe { mem::transmute(ll::rocks_thread_status_get_state_type(self.raw)) }
    }
}
