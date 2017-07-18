//! `EventListener` class contains a set of call-back functions that will
//! be called when specific RocksDB event happens such as flush.

use error::Status;
use db::DBRef;
use types::SequenceNumber;
use table_properties::{TableProperties, TablePropertiesCollection};
use options::CompressionType;
use compaction_job_stats::CompactionJobStats;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TableFileCreationReason {
    Flush,
    Compaction,
    Recovery,
}

pub struct TableFileCreationBriefInfo<'a> {
    /// the name of the database where the file was created
    pub db_name: &'a str,
    /// the name of the column family where the file was created.
    pub cf_name: &'a str,
    /// the path to the created file.
    pub file_path: &'a str,
    /// the id of the job (which could be flush or compaction) that
    /// created the file.
    pub job_id: i32,
    /// reason of creating the table.
    pub reason: TableFileCreationReason,
}

pub struct TableFileCreationInfo<'a> {
    brief_info: TableFileCreationBriefInfo<'a>,
    /// the size of the file.
    pub file_size: u64,
    /// Detailed properties of the created file.
    pub table_properties: TableProperties<'a>,
    /// The status indicating whether the creation was successful or not.
    pub status: Status,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
    /// `DB::SuggestCompactRange()` marked files for compaction
    FilesMarkedForCompaction,
}


pub struct TableFileDeletionInfo<'a> {
    /// The name of the database where the file was deleted.
    pub db_name: &'a str,
    /// The path to the deleted file.
    pub file_path: &'a str,
    /// The id of the job which deleted the file.
    pub job_id: i32,
    /// The status indicating whether the deletion was successful or not.
    pub status: Status,
}


pub struct FlushJobInfo<'a> {
    /// the name of the column family
    pub cf_name: &'a str,
    /// the path to the newly created file
    pub file_path: &'a str,
    /// the id of the thread that completed this flush job.
    pub thread_id: u64,
    /// the job id, which is unique in the same thread.
    pub job_id: i32,
    /// If true, then rocksdb is currently slowing-down all writes to prevent
    /// creating too many Level 0 files as compaction seems not able to
    /// catch up the write request speed.  This indicates that there are
    /// too many files in Level 0.
    pub triggered_writes_slowdown: bool,
    /// If true, then rocksdb is currently blocking any writes to prevent
    /// creating more L0 files.  This indicates that there are too many
    /// files in level 0.  Compactions should try to compact L0 files down
    /// to lower levels as soon as possible.
    pub triggered_writes_stop: bool,
    /// The smallest sequence number in the newly created file
    pub smallest_seqno: SequenceNumber,
    /// The largest sequence number in the newly created file
    pub largest_seqno: SequenceNumber,
    /// Table properties of the table being flushed
    pub table_properties: TableProperties<'a>,
}

pub struct CompactionJobInfo<'a> {
    /// the name of the column family where the compaction happened.
    pub cf_name: &'a str,
    /// the status indicating whether the compaction was successful or not.
    pub status: Status,
    /// the id of the thread that completed this compaction job.
    pub thread_id: u64,
    /// the job id, which is unique in the same thread.
    pub job_id: i32,
    /// the smallest input level of the compaction.
    pub base_input_level: i32,
    /// the output level of the compaction.
    pub output_level: i32,
    /// the names of the compaction input files.
    pub input_files: Vec<&'a str>,

    /// the names of the compaction output files.
    pub output_files: Vec<&'a str>,
    /// Table properties for input and output tables.
    /// The map is keyed by values from input_files and output_files.
    pub table_properties: TablePropertiesCollection,

    /// Reason to run the compaction
    pub compaction_reason: CompactionReason,

    /// Compression algorithm used for output files
    pub compression: CompressionType,

    /// If non-null, this variable stores detailed information
    /// about this compaction.
    pub stats: CompactionJobStats<'a>,
}


pub struct MemTableInfo<'a> {
    /// the name of the column family to which memtable belongs
    pub cf_name: &'a str,
    /// Sequence number of the first element that was inserted
    /// into the memtable.
    pub first_seqno: SequenceNumber,
    /// Sequence number that is guaranteed to be smaller than or equal
    /// to the sequence number of any key that could be inserted into this
    /// memtable. It can then be assumed that any write with a larger(or equal)
    /// sequence number will be present in this memtable or a later memtable.
    pub earliest_seqno: SequenceNumber,
    /// Total number of entries in memtable
    pub num_entries: u64,
    /// Total number of deletes in memtable
    pub num_deletes: u64,
}

pub struct ExternalFileIngestionInfo<'a> {
    /// the name of the column family
    pub cf_name: &'a str,
    /// Path of the file outside the DB
    pub external_file_path: &'a str,
    /// Path of the file inside the DB
    pub internal_file_path: &'a str,
    /// The global sequence number assigned to keys in this file
    pub global_seqno: SequenceNumber,
    /// Table properties of the table being flushed
    pub table_properties: TableProperties<'a>,
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CompactionListenerValueType {
    Value,
    MergeOperand,
    Delete,
    SingleDelete,
    RangeDelete,
    Invalid,
}

// A call-back function to RocksDB which will be called when the compaction
// iterator is compacting values. It is mean to be returned from
// EventListner::GetCompactionEventListner() at the beginning of compaction
// job.
pub trait CompactionEventListener {
    fn on_compaction(
        &mut self,
        level: i32,
        key: &[u8],
        value_type: CompactionListenerValueType,
        existing_value: &[u8],
        sn: SequenceNumber,
        is_new: bool,
    );
}

/// `EventListener` class contains a set of call-back functions that will
/// be called when specific RocksDB event happens such as flush.  It can
/// be used as a building block for developing custom features such as
/// stats-collector or external compaction algorithm.
///
/// Note that call-back functions should not run for an extended period of
/// time before the function returns, otherwise RocksDB may be blocked.
/// For example, it is not suggested to do `DB::CompactFiles()` (as it may
/// run for a long while) or issue many of `DB::Put()` (as Put may be blocked
/// in certain cases) in the same thread in the `EventListener` callback.
/// However, doing `DB::CompactFiles()` and `DB::Put()` in another thread is
/// considered safe.
///
/// [Threading] All `EventListener` callback will be called using the
/// actual thread that involves in that specific event.   For example, it
/// is the RocksDB background flush thread that does the actual flush to
/// call `EventListener::OnFlushCompleted()`.
///
/// [Locking] All `EventListener` callbacks are designed to be called without
/// the current thread holding any DB mutex. This is to prevent potential
/// deadlock and performance issue when using EventListener callback
/// in a complex way. However, all `EventListener` call-back functions
/// should not run for an extended period of time before the function
/// returns, otherwise RocksDB may be blocked. For example, it is not
/// suggested to do `DB::CompactFiles()` (as it may run for a long while)
/// or issue many of `DB::Put()` (as Put may be blocked in certain cases)
/// in the same thread in the `EventListener` callback. However, doing
/// `DB::CompactFiles()` and `DB::Put()` in a thread other than the
/// EventListener callback thread is considered safe.
///
/// FIXME: how to hold DB ref and CFHandle ref
pub trait EventListener {
    /// A call-back function to RocksDB which will be called whenever a
    /// registered RocksDB flushes a file.  The default implementation is
    /// no-op.
    ///
    /// Note that the this function must be implemented in a way such that
    /// it should not run for an extended period of time before the function
    /// returns.  Otherwise, RocksDB may be blocked.
    fn on_flush_completed(&mut self, db: &DBRef, flush_job_info: &FlushJobInfo) {}

    /// A call-back function to RocksDB which will be called before a
    /// RocksDB starts to flush memtables.  The default implementation is
    /// no-op.
    ///
    /// Note that the this function must be implemented in a way such that
    /// it should not run for an extended period of time before the function
    /// returns.  Otherwise, RocksDB may be blocked.
    fn on_flush_begin(&mut self, db: &DBRef, flush_job_info: &FlushJobInfo) {}

    /// A call-back function for RocksDB which will be called whenever
    /// a SST file is deleted.  Different from OnCompactionCompleted and
    /// OnFlushCompleted, this call-back is designed for external logging
    /// service and thus only provide string parameters instead
    /// of a pointer to DB.  Applications that build logic basic based
    /// on file creations and deletions is suggested to implement
    /// OnFlushCompleted and OnCompactionCompleted.
    ///
    /// Note that if applications would like to use the passed reference
    /// outside this function call, they should make copies from the
    /// returned value.
    fn on_table_file_deleted(&mut self, info: &TableFileDeletionInfo) {}

    /// A call-back function for RocksDB which will be called whenever
    /// a registered RocksDB compacts a file. The default implementation
    /// is a no-op.
    ///
    /// Note that this function must be implemented in a way such that
    /// it should not run for an extended period of time before the function
    /// returns. Otherwise, RocksDB may be blocked.
    ///
    /// @param db a pointer to the rocksdb instance which just compacted
    ///   a file.
    /// @param ci a reference to a CompactionJobInfo struct. 'ci' is released
    ///  after this function is returned, and must be copied if it is needed
    ///  outside of this function.
    fn on_compaction_completed(&mut self, db: &DBRef, ci: &CompactionJobInfo) {}

    /// A call-back function for RocksDB which will be called whenever
    /// a SST file is created.  Different from OnCompactionCompleted and
    /// OnFlushCompleted, this call-back is designed for external logging
    /// service and thus only provide string parameters instead
    /// of a pointer to DB.  Applications that build logic basic based
    /// on file creations and deletions is suggested to implement
    /// OnFlushCompleted and OnCompactionCompleted.
    ///
    /// Historically it will only be called if the file is successfully created.
    /// Now it will also be called on failure case. User can check info.status
    /// to see if it succeeded or not.
    ///
    /// Note that if applications would like to use the passed reference
    /// outside this function call, they should make copies from these
    /// returned value.
    fn on_table_file_created(&mut self, info: &TableFileCreationInfo) {}

    /// A call-back function for RocksDB which will be called before
    /// a SST file is being created. It will follow by OnTableFileCreated after
    /// the creation finishes.
    ///
    /// Note that if applications would like to use the passed reference
    /// outside this function call, they should make copies from these
    /// returned value.
    fn on_table_file_creation_started(&mut self, info: &TableFileCreationBriefInfo) {}

    /// A call-back function for RocksDB which will be called before
    /// a memtable is made immutable.
    ///
    /// Note that the this function must be implemented in a way such that
    /// it should not run for an extended period of time before the function
    /// returns.  Otherwise, RocksDB may be blocked.
    ///
    /// Note that if applications would like to use the passed reference
    /// outside this function call, they should make copies from these
    /// returned value.
    fn on_memtable_sealed(&mut self, info: &MemTableInfo) {}

    // A call-back function for RocksDB which will be called before
    // a column family handle is deleted.
    //
    // Note that the this function must be implemented in a way such that
    // it should not run for an extended period of time before the function
    // returns.  Otherwise, RocksDB may be blocked.
    // @param handle is a pointer to the column family handle to be deleted
    // which will become a dangling pointer after the deletion.
    // pub fn on_column_family_handle_deletion_started(&mut self, handle: *mut ()) {}

    /// A call-back function for RocksDB which will be called after an external
    /// file is ingested using IngestExternalFile.
    ///
    /// Note that the this function will run on the same thread as
    /// IngestExternalFile(), if this function is blocked, IngestExternalFile()
    /// will be blocked from finishing.
    fn on_external_file_ingested(&mut self, db: &DBRef, info: &ExternalFileIngestionInfo) {}

    /// Factory method to return CompactionEventListener. If multiple listeners
    /// provides CompactionEventListner, only the first one will be used.
    fn get_compaction_event_listener(&mut self) -> Option<Box<CompactionEventListener>> {
        None
    }
}
