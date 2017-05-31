//! EventListener class contains a set of call-back functions that will
//! be called when specific RocksDB event happens such as flush.

pub enum TableFileCreationReason {
    Flush,
    Compaction,
    Recovery,
}

pub enum CompactionReason {
    Unknown,
    /// [Level] number of L0 files > level0_file_num_compaction_trigger
    LevelL0FilesNum,
    /// [Level] total size of level > MaxBytesForLevel()
    LevelMaxLevelSize,
    /// [Universal] Compacting for size amplification
    UniversalSizeAmplification,
    /// [Universal] Compacting for size ratio
    UniversalSizeRatio,
    /// [Universal] number of sorted runs > level0_file_num_compaction_trigger
    UniversalSortedRunNum,
    /// [FIFO] total size > max_table_files_size
    FIFOMaxSize,
    /// Manual compaction
    ManualCompaction,
    /// DB::SuggestCompactRange() marked files for compaction
    FilesMarkedForCompaction,
}


/// EventListener class contains a set of call-back functions that will
/// be called when specific RocksDB event happens such as flush.  It can
/// be used as a building block for developing custom features such as
/// stats-collector or external compaction algorithm.
///
/// Note that call-back functions should not run for an extended period of
/// time before the function returns, otherwise RocksDB may be blocked.
/// For example, it is not suggested to do DB::CompactFiles() (as it may
/// run for a long while) or issue many of DB::Put() (as Put may be blocked
/// in certain cases) in the same thread in the EventListener callback.
/// However, doing DB::CompactFiles() and DB::Put() in another thread is
/// considered safe.
///
/// [Threading] All EventListener callback will be called using the
/// actual thread that involves in that specific event.   For example, it
/// is the RocksDB background flush thread that does the actual flush to
/// call EventListener::OnFlushCompleted().
///
/// [Locking] All EventListener callbacks are designed to be called without
/// the current thread holding any DB mutex. This is to prevent potential
/// deadlock and performance issue when using EventListener callback
/// in a complex way. However, all EventListener call-back functions
/// should not run for an extended period of time before the function
/// returns, otherwise RocksDB may be blocked. For example, it is not
/// suggested to do DB::CompactFiles() (as it may run for a long while)
/// or issue many of DB::Put() (as Put may be blocked in certain cases)
/// in the same thread in the EventListener callback. However, doing
/// DB::CompactFiles() and DB::Put() in a thread other than the
/// EventListener callback thread is considered safe.
pub struct EventListener;
