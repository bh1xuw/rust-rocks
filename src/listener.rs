//! `EventListener` class contains a set of call-back functions that will
//! be called when specific RocksDB event happens such as flush.

use rocks_sys as ll;

use std::marker::PhantomData;
use std::ptr;
use std::str;
use std::slice;
use std::mem;
use std::fmt;

use error::Status;
use db::DBRef;
use types::SequenceNumber;
use table_properties::{TableProperties, TablePropertiesCollection};
use options::CompressionType;
use compaction_job_stats::CompactionJobStats;
use to_raw::FromRaw;

use super::Result;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TableFileCreationReason {
    Flush,
    Compaction,
    Recovery,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct TableFileCreationInfo<'a> {
    brief_info: TableFileCreationBriefInfo<'a>,
    /// the size of the file.
    pub file_size: u64,
    /// Detailed properties of the created file.
    pub table_properties: TableProperties<'a>,
    /// The status indicating whether the creation was successful or not.
    pub status: Result<()>,
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

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BackgroundErrorReason {
    Flush,
    Compaction,
    WriteCallback,
    MemTable,
}

#[derive(Debug)]
pub struct TableFileDeletionInfo<'a> {
    /// The name of the database where the file was deleted.
    pub db_name: &'a str,
    /// The path to the deleted file.
    pub file_path: &'a str,
    /// The id of the job which deleted the file.
    pub job_id: i32,
    /// The status indicating whether the deletion was successful or not.
    pub status: Result<()>,
}

#[derive(Debug)]
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

// Big struct, avoid expensive building
pub struct CompactionJobInfo<'a> {
    raw: *mut ll::rocks_compaction_job_info_t,
    _marker: PhantomData<&'a ()>,
}

impl<'a> fmt::Debug for CompactionJobInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CompactionJobInfo")
            .field("cf_name", &self.cf_name())
            .field("status", &self.status())
            .field("inputs", &self.input_files().len())
            .field("outputs", &self.output_files().len())
            .finish()
    }
}

impl<'a> CompactionJobInfo<'a> {
    /// the name of the column family where the compaction happened.
    pub fn cf_name(&self) -> &'a str {
        let mut len = 0;
        unsafe {
            let ptr = ll::rocks_compaction_job_info_get_cf_name(self.raw, &mut len);
            str::from_utf8_unchecked(slice::from_raw_parts(ptr as *const u8, len))
        }
    }

    /// the status indicating whether the compaction was successful or not.
    pub fn status(&self) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_compaction_job_info_get_status(self.raw, &mut status);
            Result::from_ll(status)
        }
    }

    /// the id of the thread that completed this compaction job.
    pub fn thread_id(&self) -> u64 {
        unsafe { ll::rocks_compaction_job_info_get_thread_id(self.raw) }
    }

    /// the job id, which is unique in the same thread.
    pub fn job_id(&self) -> i32 {
        unsafe { ll::rocks_compaction_job_info_get_job_id(self.raw) as i32 }
    }

    /// the smallest input level of the compaction.
    pub fn base_input_level(&self) -> i32 {
        unsafe { ll::rocks_compaction_job_info_get_base_input_level(self.raw) as i32 }
    }

    /// the output level of the compaction.
    pub fn output_level(&self) -> i32 {
        unsafe { ll::rocks_compaction_job_info_get_output_level(self.raw) as i32 }
    }

    /// the names of the compaction input files.
    pub fn input_files(&self) -> Vec<&'a str> {
        unsafe {
            let num = ll::rocks_compaction_job_info_get_input_files_num(self.raw);
            let mut ptrs = vec![ptr::null(); num];
            let mut sizes = vec![0_usize; num];
            ll::rocks_compaction_job_info_get_input_files(self.raw, ptrs.as_mut_ptr(), sizes.as_mut_ptr());
            ptrs.iter()
                .zip(sizes.iter())
                .map(|(&ptr, &len)| str::from_utf8_unchecked(slice::from_raw_parts(ptr as *const u8, len)))
                .collect()
        }
    }

    /// the names of the compaction output files.
    pub fn output_files(&self) -> Vec<&'a str> {
        unsafe {
            let num = ll::rocks_compaction_job_info_get_output_files_num(self.raw);
            let mut ptrs = vec![ptr::null(); num];
            let mut sizes = vec![0_usize; num];
            ll::rocks_compaction_job_info_get_output_files(self.raw, ptrs.as_mut_ptr(), sizes.as_mut_ptr());
            ptrs.iter()
                .zip(sizes.iter())
                .map(|(&ptr, &len)| str::from_utf8_unchecked(slice::from_raw_parts(ptr as *const u8, len)))
                .collect()
        }
    }

    /// Table properties for input and output tables.
    /// The map is keyed by values from input_files and output_files.
    pub fn table_properties(&self) -> TablePropertiesCollection {
        unsafe { TablePropertiesCollection::from_ll(ll::rocks_compaction_job_info_get_table_properties(self.raw)) }
    }

    /// Reason to run the compaction
    pub fn compaction_reason(&self) -> CompactionReason {
        unsafe { mem::transmute(ll::rocks_compaction_job_info_get_compaction_reason(self.raw)) }
    }

    /// Compression algorithm used for output files
    pub fn compression(&self) -> CompressionType {
        unsafe { mem::transmute(ll::rocks_compaction_job_info_get_compression(self.raw)) }
    }

    /// If non-null, this variable stores detailed information
    /// about this compaction.
    pub fn stats(&self) -> CompactionJobStats {
        unsafe { CompactionJobStats::from_ll(ll::rocks_compaction_job_info_get_stats(self.raw)) }
    }
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
// `EventListner::GetCompactionEventListner()` at the beginning of compaction
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

    /// A call-back function for RocksDB which will be called before setting the
    /// background error status to a non-OK value. The new background error status
    /// is provided in `bg_error` and can be modified by the callback. E.g., a
    /// callback can suppress errors by resetting it to Status::OK(), thus
    /// preventing the database from entering read-only mode. We do not provide any
    /// guarantee when failed flushes/compactions will be rescheduled if the user
    /// suppresses an error.
    ///
    /// Note that this function can run on the same threads as flush, compaction,
    /// and user writes. So, it is extremely important not to perform heavy
    /// computations or blocking calls in this function.
    fn on_background_error(&mut self, reason: BackgroundErrorReason, bg_error: Status) {}

    /// Factory method to return CompactionEventListener. If multiple listeners
    /// provides CompactionEventListner, only the first one will be used.
    fn get_compaction_event_listener(&mut self) -> Option<Box<CompactionEventListener>> {
        None
    }
}


#[doc(hidden)]
pub mod c {
    use std::str;
    use std::slice;
    use std::mem;
    use std::ptr;
    use super::*;
    use db::DBRef;
    use to_raw::FromRaw;


    unsafe fn flush_job_info_convert<'a>(info: *mut ll::rocks_flush_job_info_t) -> FlushJobInfo<'a> {
        FlushJobInfo {
            cf_name: {
                let mut len = 0;
                let ptr = ll::rocks_flush_job_info_get_cf_name(info, &mut len);
                str::from_utf8_unchecked(slice::from_raw_parts(ptr as *const u8, len))
            },
            file_path: {
                let mut len = 0;
                let ptr = ll::rocks_flush_job_info_get_file_path(info, &mut len);
                str::from_utf8_unchecked(slice::from_raw_parts(ptr as *const u8, len))
            },
            thread_id: ll::rocks_flush_job_info_get_thread_id(info),
            job_id: ll::rocks_flush_job_info_get_job_id(info) as i32,
            triggered_writes_slowdown: ll::rocks_flush_job_info_get_triggered_writes_slowdown(info) != 0,
            triggered_writes_stop: ll::rocks_flush_job_info_get_triggered_writes_stop(info) != 0,
            smallest_seqno: SequenceNumber(ll::rocks_flush_job_info_get_smallest_seqno(info)),
            largest_seqno: SequenceNumber(ll::rocks_flush_job_info_get_largest_seqno(info)),
            table_properties: TableProperties::from_ll(ll::rocks_flush_job_info_get_table_properties(info)),
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_event_listener_on_flush_completed(
        l: *mut (),
        db: *mut (), // DB**
        info: *mut ll::rocks_flush_job_info_t,
    ) {
        let listener = l as *mut Box<EventListener>;
        let db_ref = mem::transmute::<_, DBRef>(db);
        let flush_job_info = flush_job_info_convert(info);

        (*listener).on_flush_completed(&db_ref, &flush_job_info);
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_event_listener_on_flush_begin(
        l: *mut (),
        db: *mut (), // DB**
        info: *mut ll::rocks_flush_job_info_t,
    ) {
        let listener = l as *mut Box<EventListener>;
        let db_ref = mem::transmute::<_, DBRef>(db);
        let flush_job_info = flush_job_info_convert(info);

        (*listener).on_flush_begin(&db_ref, &flush_job_info);
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_event_listener_on_table_file_deleted(
        l: *mut (),
        info: *mut ll::rocks_table_file_deletion_info_t,
    ) {
        let listener = l as *mut Box<EventListener>;
        let info = TableFileDeletionInfo {
            db_name: {
                let mut len = 0;
                let ptr = ll::rocks_table_file_deletion_info_get_db_name(info, &mut len);
                str::from_utf8_unchecked(slice::from_raw_parts(ptr as *const u8, len))
            },
            file_path: {
                let mut len = 0;
                let ptr = ll::rocks_table_file_deletion_info_get_file_path(info, &mut len);
                str::from_utf8_unchecked(slice::from_raw_parts(ptr as *const u8, len))
            },
            job_id: ll::rocks_table_file_deletion_info_get_job_id(info) as i32,
            status: {
                let mut status = ptr::null_mut::<ll::rocks_status_t>();
                ll::rocks_table_file_deletion_info_get_status(info, &mut status);
                Result::from_ll(status)
            },
        };

        (*listener).on_table_file_deleted(&info);
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_event_listener_on_compaction_completed(
        l: *mut (),
        db: *mut (), // DB** <=> DBRef
        ci: *mut ll::rocks_compaction_job_info_t,
    ) {
        let listener = l as *mut Box<EventListener>;
        let db_ref = mem::transmute::<_, DBRef>(db);
        let info = CompactionJobInfo {
            raw: ci,
            _marker: PhantomData,
        };

        (*listener).on_compaction_completed(&db_ref, &info);
    }



    #[no_mangle]
    pub unsafe extern "C" fn rust_event_listener_drop(l: *mut ()) {
        let listener = l as *mut Box<EventListener>;
        Box::from_raw(listener);
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::rocksdb::*;

    #[derive(Default)]
    struct MyEventListener {
        flush_completed_called: usize,
        flush_begin_called: usize,
        table_file_deleted_called: usize,
        compaction_completed_called: usize,
    }

    impl Drop for MyEventListener {
        fn drop(&mut self) {
            assert!(
                self.flush_begin_called * self.flush_completed_called * self.table_file_deleted_called *
                    self.compaction_completed_called > 0
            );

            // assert!(false);
            // FIXME: must assert drop is called
        }
    }

    impl EventListener for MyEventListener {
        fn on_flush_completed(&mut self, db: &DBRef, flush_job_info: &FlushJobInfo) {
            assert!(db.name().len() > 0, "DB name is accessable");
            self.flush_completed_called += 1;
        }

        fn on_flush_begin(&mut self, db: &DBRef, flush_job_info: &FlushJobInfo) {
            self.flush_begin_called += 1;
        }

        fn on_table_file_deleted(&mut self, info: &TableFileDeletionInfo) {
            assert!(info.status.is_ok());
            self.table_file_deleted_called += 1;
        }

        fn on_compaction_completed(&mut self, db: &DBRef, ci: &CompactionJobInfo) {
            // println!("compation => {:?}: {:?}", ci.cf_name(), ci.input_files());
            assert!(ci.status().is_ok());
            assert!(ci.stats().num_input_files() > 0);
            self.compaction_completed_called += 1;
        }
    }


    #[test]
    fn event_listener_works() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(
            Options::default().map_db_options(|db| {
                db.create_if_missing(true).add_listener(
                    MyEventListener::default(),
                )
            }),
            &tmp_dir,
        ).unwrap();

        for i in 0..100 {
            let key = format!("test2-key-{}", i);
            let val = format!("rocksdb-value-{}", i * 10);

            db.put(&WriteOptions::default(), key.as_bytes(), val.as_bytes())
                .unwrap();

            if i % 6 == 0 {
                assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
            }
            if i % 36 == 0 {
                assert!(
                    db.compact_range(&CompactRangeOptions::default(), ..)
                        .is_ok()
                );
            }
        }

        assert!(db.flush(&Default::default()).is_ok());
    }
}
