//! WriteBatch holds a collection of updates to apply atomically to a DB.
//!
//! The updates are applied in the order in which they are added
//! to the WriteBatch.  For example, the value of "key" will be "v3"
//! after the following batch is written:
//!
//!    batch.Put("key", "v1");
//!    batch.Delete("key");
//!    batch.Put("key", "v2");
//!    batch.Put("key", "v3");
//!
//! Multiple threads can invoke const methods on a WriteBatch without
//! external synchronization, but if any of the threads may call a
//! non-const method, all threads accessing the same WriteBatch must use
//! external synchronization.

use std::mem;

use rocks_sys as ll;

use status::Status;
use db::ColumnFamilyHandle;

pub struct WriteBatch {
    raw: *mut ll::rocks_writebatch_t,
}

impl Drop for WriteBatch {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_writebatch_destroy(self.raw)
        }
    }
}

impl WriteBatch {
    pub fn new() -> WriteBatch {
        WriteBatch{
            raw: unsafe { ll::rocks_writebatch_create() },
        }
    }

    pub fn raw(&self) -> *mut ll::rocks_writebatch_t {
        self.raw
    }

    /// Clear all updates buffered in this batch.
    pub fn clear(&mut self) {
        unsafe {
            ll::rocks_writebatch_clear(self.raw);
        }
    }

    /// Store the mapping "key->value" in the database.
    pub fn put(self, key: &[u8], value: &[u8]) -> Self {
        unsafe {
            ll::rocks_writebatch_put(self.raw,
                                     key.as_ptr() as _, key.len(),
                                     value.as_ptr() as _, value.len());
        }
        self
    }

    pub fn put_cf(self, column_family: &ColumnFamilyHandle,
                  key: &[u8], value: &[u8]) -> Self {
        unsafe {
            ll::rocks_writebatch_put_cf(self.raw,
                                        column_family.raw(),
                                        key.as_ptr() as _, key.len(),
                                        value.as_ptr() as _, value.len());
        }
        self
    }

    /// Variant of Put() that gathers output like writev(2).  The key and value
    /// that will be written to the database are concatentations of arrays of
    /// slices.
    pub fn putv(&mut self, key: &[&[u8]], value: &[&[u8]]) {
        unimplemented!()
    }

    pub fn putv_cf(&mut self, column_family: &ColumnFamilyHandle,
                  key: &[&[u8]], value: &[&[u8]]) {
        unimplemented!()
    }

    /// If the database contains a mapping for "key", erase it.  Else do nothing.
    pub fn delete(self, key: &[u8]) -> Self {
        unsafe {
            ll::rocks_writebatch_delete(self.raw,
                                     key.as_ptr() as _, key.len());
        }
        self
    }

    pub fn delete_cf(self, column_family: &ColumnFamilyHandle, key: &[u8]) -> Self {
        unsafe {
            ll::rocks_writebatch_delete_cf(self.raw,
                                           column_family.raw(),
                                           key.as_ptr() as _, key.len());
        }
        self
    }

    /// variant that takes SliceParts
    pub fn deletev(&mut self, key: &[&[u8]]) {
        unimplemented!()
    }

    pub fn deletev_cf(&mut self, column_family: &ColumnFamilyHandle, key: &[&[u8]]) {
        unimplemented!()
    }

    /// WriteBatch implementation of DB::SingleDelete().  See db.h.
    pub fn single_delete(self, key: &[u8]) -> Self {
        unsafe {
            ll::rocks_writebatch_single_delete(self.raw,
                                        key.as_ptr() as _, key.len());
        }
        self
    }

    pub fn single_delete_cf(self, column_family: &ColumnFamilyHandle, key: &[u8]) -> Self {
        unsafe {
            ll::rocks_writebatch_single_delete_cf(self.raw,
                                                  column_family.raw(),
                                                  key.as_ptr() as _, key.len());
        }
        self
    }

    /// variant that takes SliceParts
    pub fn single_deletev(&mut self, key: &[&[u8]]) {
        unimplemented!()
    }

    pub fn single_deletev_cf(&mut self, column_family: &ColumnFamilyHandle, key: &[&[u8]]) {
        unimplemented!()
    }

    /// WriteBatch implementation of DB::DeleteRange().  See db.h.
    pub fn delete_range(self, begin_key: &[u8], end_key: &[u8]) -> Self {
        unsafe {
            ll::rocks_writebatch_delete_range(self.raw,
                                              begin_key.as_ptr() as _, begin_key.len(),
                                              end_key.as_ptr() as _, end_key.len());
        }
        self
    }

    pub fn delete_range_cf(self, column_family: &ColumnFamilyHandle, begin_key: &[u8], end_key: &[u8]) -> Self {
        unsafe {
            ll::rocks_writebatch_delete_range_cf(self.raw,
                                                 column_family.raw(),
                                                 begin_key.as_ptr() as _, begin_key.len(),
                                                 end_key.as_ptr() as _, end_key.len());
        }
        self
    }

    /// variant that takes SliceParts
    pub fn deletev_range(&mut self, begin_key: &[&[u8]], end_key: &[&[u8]]) {
        unimplemented!()
    }

    pub fn deletev_range_cf(&mut self, column_family: &ColumnFamilyHandle, begin_key: &[&[u8]], end_key: &[&[u8]]) {
        unimplemented!()
    }


    /// Merge "value" with the existing value of "key" in the database.
    /// "key->merge(existing, value)"
    pub fn merge(self, key: &[u8], value: &[u8]) -> Self {
        unsafe {
            ll::rocks_writebatch_merge(self.raw,
                                       key.as_ptr() as _, key.len(),
                                       value.as_ptr() as _, value.len());
        }
        self
    }

    pub fn merge_cf(self, column_family: &ColumnFamilyHandle,
                    key: &[u8], value: &[u8]) -> Self {
        unsafe {
            ll::rocks_writebatch_merge_cf(self.raw,
                                          column_family.raw(),
                                          key.as_ptr() as _, key.len(),
                                          value.as_ptr() as _, value.len());
        }
        self
    }

    // variant that takes SliceParts
    pub fn mergev(&mut self, key: &[&[u8]], value: &[&[u8]]) {
        unimplemented!()
    }

    pub fn mergev_cf(&mut self, column_family: &ColumnFamilyHandle,
                     key: &[&[u8]], value: &[&[u8]]) {
        unimplemented!()
    }

    /// Append a blob of arbitrary size to the records in this batch. The blob will
    /// be stored in the transaction log but not in any other file. In particular,
    /// it will not be persisted to the SST files. When iterating over this
    /// WriteBatch, WriteBatch::Handler::LogData will be called with the contents
    /// of the blob as it is encountered. Blobs, puts, deletes, and merges will be
    /// encountered in the same order in thich they were inserted. The blob will
    /// NOT consume sequence number(s) and will NOT increase the count of the batch
    ///
    /// Example application: add timestamps to the transaction log for use in
    /// replication.
    pub fn put_log_data(self, blob: &[u8]) -> Self {
        unsafe {
            ll::rocks_writebatch_put_log_data(self.raw,
                                              blob.as_ptr() as _, blob.len());
        }
        self
    }

    /// Records the state of the batch for future calls to RollbackToSavePoint().
    /// May be called multiple times to set multiple save points.
    pub fn set_save_point(self) -> Self {
        unsafe {
            ll::rocks_writebatch_set_save_point(self.raw);
        }
        self
    }

    /// Remove all entries in this batch (Put, Merge, Delete, PutLogData) since the
    /// most recent call to SetSavePoint() and removes the most recent save point.
    /// If there is no previous call to SetSavePoint(), Status::NotFound()
    /// will be returned.
    /// Otherwise returns Status::OK().
    pub fn rollback_to_save_point(self) -> Self {
        unsafe {
            let mut status = mem::zeroed();
            ll::rocks_writebatch_rollback_to_save_point(self.raw, &mut status);
        }
        self
    }


    // Retrieve the serialized version of this batch.
    // Data()
    pub fn get_data(&self) -> &[u8] {
        unimplemented!()
    }

    // Retrieve data size of the batch.
    pub fn get_data_size(&self) -> usize {
        0
    }

    // Returns the number of updates in the batch
    pub fn count(&self) -> usize {
        unsafe {
            ll::rocks_writebatch_count(self.raw) as usize
        }
    }

    // Returns true if PutCF will be called during Iterate
    pub fn has_put(&self) -> bool {
        false
    }

    // Returns true if DeleteCF will be called during Iterate
    pub fn has_delete(&self) -> bool {
        false
    }

    // Returns true if SingleDeleteCF will be called during Iterate
    pub fn has_single_delete(&self) -> bool {
        false
    }

    // Returns true if DeleteRangeCF will be called during Iterate
    pub fn has_delete_range(&self) -> bool {
        false
    }

    // Returns true if MergeCF will be called during Iterate
    pub fn has_merge(&self) -> bool {
        false
    }

    // Returns true if MarkBeginPrepare will be called during Iterate
    pub fn has_begin_prepare(&self) -> bool {
        false
    }

    // Returns true if MarkEndPrepare will be called during Iterate
    pub fn has_end_prepare(&self) -> bool {
        false
    }

    // Returns trie if MarkCommit will be called during Iterate
    pub fn has_commit(&self) -> bool {
        false
    }

    // Returns trie if MarkRollback will be called during Iterate
    pub fn has_rollback(&self) -> bool {
        false
    }

    // marks this point in the WriteBatch as the last record to
    // be inserted into the WAL, provided the WAL is enabled
    //void MarkWalTerminationPoint();
    //const SavePoint& GetWalTerminationPoint() const { return wal_term_point_; }
}


#[test]
fn test_write_batch_create() {
    let batch = WriteBatch::new()
        .put(b"name", b"rocksdb");
    assert!(batch.count() == 1);
    let batch = batch.delete(b"name");
    assert_eq!(batch.count(), 2);
}

