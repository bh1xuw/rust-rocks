/// This file defines the structures for exposing run-time status of any
/// rocksdb-related thread.  Such run-time status can be obtained via
/// GetThreadList() API.
///
/// Note that all thread-status features are still under-development, and
/// thus APIs and class definitions might subject to change at this point.
/// Will remove this comment once the APIs have been finalized.

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
#[derive(Debug)]
#[repr(C)]
pub struct ThreadStatus {
    raw: *mut (),
}

// TODO: wait APIs to be finalized.
