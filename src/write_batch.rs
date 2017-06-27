//! `WriteBatch` holds a collection of updates to apply atomically to a DB.
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

use std::fmt;
use std::slice;
use std::ptr;
use std::os::raw::{c_uchar, c_void};

use rocks_sys as ll;

use db::ColumnFamilyHandle;
use to_raw::{ToRaw, FromRaw};
use super::Result;

/// `WriteBatch` holds a collection of updates to apply atomically to a DB.
pub struct WriteBatch {
    raw: *mut ll::rocks_writebatch_t,
}

impl Drop for WriteBatch {
    fn drop(&mut self) {
        unsafe { ll::rocks_writebatch_destroy(self.raw) }
    }
}

impl Clone for WriteBatch {
    fn clone(&self) -> Self {
        WriteBatch { raw: unsafe { ll::rocks_writebatch_copy(self.raw) } }
    }
}

impl fmt::Debug for WriteBatch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WriteBatch {{{:?}}}", String::from_utf8_lossy(self.get_data()))
    }
}

// FIXME: this is directly converted to raw pointer
//        not the rocks wrapped
impl ToRaw<ll::rocks_raw_writebatch_t> for WriteBatch {
    fn raw(&self) -> *mut ll::rocks_raw_writebatch_t {
        unsafe { ll::rocks_writebatch_get_writebatch(self.raw) }
    }
}

impl Default for WriteBatch {
    fn default() -> Self {
        WriteBatch::new()
    }
}

impl WriteBatch {
    pub fn new() -> WriteBatch {
        WriteBatch { raw: unsafe { ll::rocks_writebatch_create() } }
    }

    pub fn with_reserved_bytes(reserved_bytes: usize) -> WriteBatch {
        WriteBatch { raw: unsafe { ll::rocks_writebatch_create_with_reserved_bytes(reserved_bytes) } }
    }

    /// Clear all updates buffered in this batch.
    pub fn clear(&mut self) {
        unsafe {
            ll::rocks_writebatch_clear(self.raw);
        }
    }

    /// Store the mapping "key->value" in the database.
    pub fn put(&mut self, key: &[u8], value: &[u8]) -> &mut Self {
        unsafe {
            ll::rocks_writebatch_put(self.raw, key.as_ptr() as _, key.len(), value.as_ptr() as _, value.len());
        }
        self
    }

    pub fn put_cf(&mut self, column_family: &ColumnFamilyHandle, key: &[u8], value: &[u8]) -> &mut Self {
        unsafe {
            ll::rocks_writebatch_put_cf(
                self.raw,
                column_family.raw(),
                key.as_ptr() as _,
                key.len(),
                value.as_ptr() as _,
                value.len(),
            );
        }
        self
    }

    /// Variant of Put() that gathers output like writev(2).  The key and value
    /// that will be written to the database are concatentations of arrays of
    /// slices.
    pub fn putv(&mut self, key: &[&[u8]], value: &[&[u8]]) -> &mut Self {
        unimplemented!()
    }

    pub fn putv_cf(&mut self, column_family: &ColumnFamilyHandle, key: &[&[u8]], value: &[&[u8]]) -> &mut Self {
        unimplemented!()
    }

    /// If the database contains a mapping for "key", erase it.  Else do nothing.
    pub fn delete(&mut self, key: &[u8]) -> &mut Self {
        unsafe {
            ll::rocks_writebatch_delete(self.raw, key.as_ptr() as _, key.len());
        }
        self
    }

    pub fn delete_cf(&mut self, column_family: &ColumnFamilyHandle, key: &[u8]) -> &mut Self {
        unsafe {
            ll::rocks_writebatch_delete_cf(self.raw, column_family.raw(), key.as_ptr() as _, key.len());
        }
        self
    }

    /// variant that takes SliceParts
    pub fn deletev(&mut self, key: &[&[u8]]) -> &mut Self {
        unimplemented!()
    }

    pub fn deletev_cf(&mut self, column_family: &ColumnFamilyHandle, key: &[&[u8]]) -> &mut Self {
        unimplemented!()
    }

    /// WriteBatch implementation of DB::SingleDelete().  See db.h.
    pub fn single_delete(&mut self, key: &[u8]) -> &mut Self {
        unsafe {
            ll::rocks_writebatch_single_delete(self.raw, key.as_ptr() as _, key.len());
        }
        self
    }

    pub fn single_delete_cf(&mut self, column_family: &ColumnFamilyHandle, key: &[u8]) -> &mut Self {
        unsafe {
            ll::rocks_writebatch_single_delete_cf(self.raw, column_family.raw(), key.as_ptr() as _, key.len());
        }
        self
    }

    /// variant that takes SliceParts
    pub fn single_deletev(&mut self, key: &[&[u8]]) -> &mut Self {
        unimplemented!()
    }

    pub fn single_deletev_cf(&mut self, column_family: &ColumnFamilyHandle, key: &[&[u8]]) -> &mut Self {
        unimplemented!()
    }

    /// WriteBatch implementation of DB::DeleteRange().  See db.h.
    pub fn delete_range(&mut self, begin_key: &[u8], end_key: &[u8]) -> &mut Self {
        unsafe {
            ll::rocks_writebatch_delete_range(
                self.raw,
                begin_key.as_ptr() as _,
                begin_key.len(),
                end_key.as_ptr() as _,
                end_key.len(),
            );
        }
        self
    }

    pub fn delete_range_cf(
        &mut self,
        column_family: &ColumnFamilyHandle,
        begin_key: &[u8],
        end_key: &[u8],
    ) -> &mut Self {
        unsafe {
            ll::rocks_writebatch_delete_range_cf(
                self.raw,
                column_family.raw(),
                begin_key.as_ptr() as _,
                begin_key.len(),
                end_key.as_ptr() as _,
                end_key.len(),
            );
        }
        self
    }

    /// variant that takes SliceParts
    pub fn deletev_range(&mut self, begin_key: &[&[u8]], end_key: &[&[u8]]) -> &mut Self {
        unimplemented!()
    }

    pub fn deletev_range_cf(
        &mut self,
        column_family: &ColumnFamilyHandle,
        begin_key: &[&[u8]],
        end_key: &[&[u8]],
    ) -> &mut Self {
        unimplemented!()
    }


    /// Merge "value" with the existing value of "key" in the database.
    /// "key->merge(existing, value)"
    pub fn merge(&mut self, key: &[u8], value: &[u8]) -> &mut Self {
        unsafe {
            ll::rocks_writebatch_merge(self.raw, key.as_ptr() as _, key.len(), value.as_ptr() as _, value.len());
        }
        self
    }

    pub fn merge_cf(&mut self, column_family: &ColumnFamilyHandle, key: &[u8], value: &[u8]) -> &mut Self {
        unsafe {
            ll::rocks_writebatch_merge_cf(
                self.raw,
                column_family.raw(),
                key.as_ptr() as _,
                key.len(),
                value.as_ptr() as _,
                value.len(),
            );
        }
        self
    }

    // variant that takes SliceParts
    pub fn mergev(&mut self, key: &[&[u8]], value: &[&[u8]]) -> &mut Self {
        unimplemented!()
    }

    pub fn mergev_cf(&mut self, column_family: &ColumnFamilyHandle, key: &[&[u8]], value: &[&[u8]]) -> &mut Self {
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
    pub fn put_log_data(&mut self, blob: &[u8]) -> &mut Self {
        unsafe {
            ll::rocks_writebatch_put_log_data(self.raw, blob.as_ptr() as _, blob.len());
        }
        self
    }

    /// Records the state of the batch for future calls to RollbackToSavePoint().
    /// May be called multiple times to set multiple save points.
    pub fn set_save_point(&mut self) -> &mut Self {
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
    pub fn rollback_to_save_point(&mut self) -> Result<()> {
        let mut status = ptr::null_mut();
        unsafe {
            ll::rocks_writebatch_rollback_to_save_point(self.raw, &mut status);
            FromRaw::from_ll(status)
        }
    }

    /// Support for iterating over the contents of a batch.
    pub fn iterate<H: WriteBatchHandler>(&self, handler: &mut H) -> Result<()> {
        let mut status = ptr::null_mut();
        unsafe {
            let raw_ptr = Box::into_raw(Box::new(handler as &mut WriteBatchHandler)) as *mut c_void; // Box<&mut WriteBatchHandler>
            ll::rocks_writebatch_iterate(self.raw, raw_ptr, &mut status);
            FromRaw::from_ll(status)
        }
    }

    /// Retrieve the serialized version of this batch.
    pub fn get_data(&self) -> &[u8] {
        let mut size = 0;
        unsafe {
            let ptr = ll::rocks_writebatch_data(self.raw, &mut size);
            slice::from_raw_parts(ptr as *const _, size)
        }
    }

    /// Returns the number of updates in the batch
    pub fn count(&self) -> usize {
        unsafe { ll::rocks_writebatch_count(self.raw) as usize }
    }

    /// Returns true if PutCF will be called during Iterate
    pub fn has_put(&self) -> bool {
        unsafe { ll::rocks_writebatch_has_put(self.raw) != 0 }
    }

    /// Returns true if DeleteCF will be called during Iterate
    pub fn has_delete(&self) -> bool {
        unsafe { ll::rocks_writebatch_has_delete(self.raw) != 0 }
    }

    /// Returns true if SingleDeleteCF will be called during Iterate
    pub fn has_single_delete(&self) -> bool {
        unsafe { ll::rocks_writebatch_has_single_delete(self.raw) != 0 }
    }

    /// Returns true if DeleteRangeCF will be called during Iterate
    pub fn has_delete_range(&self) -> bool {
        unsafe { ll::rocks_writebatch_has_delete_range(self.raw) != 0 }
    }

    /// Returns true if MergeCF will be called during Iterate
    pub fn has_merge(&self) -> bool {
        unsafe { ll::rocks_writebatch_has_merge(self.raw) != 0 }
    }

    /// Returns true if MarkBeginPrepare will be called during Iterate
    pub fn has_begin_prepare(&self) -> bool {
        unsafe { ll::rocks_writebatch_has_begin_prepare(self.raw) != 0 }
    }

    /// Returns true if MarkEndPrepare will be called during Iterate
    pub fn has_end_prepare(&self) -> bool {
        unsafe { ll::rocks_writebatch_has_end_prepare(self.raw) != 0 }
    }

    /// Returns trie if MarkCommit will be called during Iterate
    pub fn has_commit(&self) -> bool {
        unsafe { ll::rocks_writebatch_has_commit(self.raw) != 0 }
    }

    /// Returns trie if MarkRollback will be called during Iterate
    pub fn has_rollback(&self) -> bool {
        unsafe { ll::rocks_writebatch_has_put(self.raw) != 0 }
    }
}

/// Support for iterating over the contents of a batch.
///
/// All handler functions in this class provide default implementations so
/// we won't break existing clients of Handler on a source code level when
/// adding a new member function.
pub trait WriteBatchHandler {
    fn put_cf(&mut self, column_family_id: u32, key: &[u8], value: &[u8]) {}
    fn delete_cf(&mut self, column_family_id: u32, key: &[u8]) {}
    fn single_delete_cf(&mut self, column_family_id: u32, key: &[u8]) {}
    fn delete_range_cf(&mut self, column_family_id: u32, begin_key: &[u8], end_key: &[u8]) {}
    fn merge_cf(&mut self, column_family_id: u32, key: &[u8], value: &[u8]) {}
    fn log_data(&mut self, blob: &[u8]) {}
    fn mark_begin_prepare(&mut self) {}
    fn mark_end_prepare(&mut self, xid: &[u8]) {}
    fn mark_rollback(&mut self, xid: &[u8]) {}
    fn mark_commit(&mut self, xid: &[u8]) {}
    /// Continue is called by WriteBatch::Iterate. If it returns false,
    /// iteration is halted. Otherwise, it continues iterating. The default
    /// implementation always returns true.
    fn will_continue(&mut self) -> bool {
        true
    }
}

/// Rust style WriteBatch decompose
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum WriteBatchEntry {
    Put {
        column_family_id: u32,
        key: Vec<u8>,
        value: Vec<u8>,
    },
    Delete {
        column_family_id: u32,
        key: Vec<u8>,
    },
    SingleDelete {
        column_family_id: u32,
        key: Vec<u8>,
    },
    DeleteRange {
        column_family_id: u32,
        begin_key: Vec<u8>,
        end_key: Vec<u8>,
    },
    Merge {
        column_family_id: u32,
        key: Vec<u8>,
        value: Vec<u8>,
    },
    LogData {
        blob: Vec<u8>,
    },
    BeginPrepareMark,
    EndPrepareMark {
        xid: Vec<u8>,
    },
    RollbackMark {
        xid: Vec<u8>,
    },
    CommitMark {
        xid: Vec<u8>,
    }
}


#[derive(Default, Debug)]
pub struct WriteBatchIteratorHandler {
    pub entries: Vec<WriteBatchEntry>
}

impl WriteBatchHandler for WriteBatchIteratorHandler {
    fn put_cf(&mut self, column_family_id: u32, key: &[u8], value: &[u8]) {
        self.entries.push(WriteBatchEntry::Put {
            column_family_id,
            key: key.to_owned(),
            value: value.to_owned(),
        });
    }
    fn delete_cf(&mut self, column_family_id: u32, key: &[u8]) {
        self.entries.push(WriteBatchEntry::Delete {
            column_family_id,
            key: key.to_owned(),
        });
    }
    fn single_delete_cf(&mut self, column_family_id: u32, key: &[u8]) {
        self.entries.push(WriteBatchEntry::SingleDelete {
            column_family_id,
            key: key.to_owned(),
        });
    }
    fn delete_range_cf(&mut self, column_family_id: u32, begin_key: &[u8], end_key: &[u8]) {
        self.entries.push(WriteBatchEntry::DeleteRange {
            column_family_id,
            begin_key: begin_key.to_owned(),
            end_key: end_key.to_owned(),
        });
    }
    fn merge_cf(&mut self, column_family_id: u32, key: &[u8], value: &[u8]) {
        self.entries.push(WriteBatchEntry::Merge {
            column_family_id,
            key: key.to_owned(),
            value: value.to_owned(),
        });
    }
    fn log_data(&mut self, blob: &[u8]) {
        self.entries.push(WriteBatchEntry::LogData {
            blob: blob.to_owned(),
        });
    }
    fn mark_begin_prepare(&mut self) {
        self.entries.push(WriteBatchEntry::BeginPrepareMark);
    }
    fn mark_end_prepare(&mut self, xid: &[u8]) {
        self.entries.push(WriteBatchEntry::EndPrepareMark {
            xid: xid.to_owned(),
        });
    }
    fn mark_rollback(&mut self, xid: &[u8]) {
        self.entries.push(WriteBatchEntry::RollbackMark {
            xid: xid.to_owned(),
        });
    }
    fn mark_commit(&mut self, xid: &[u8]) {
        self.entries.push(WriteBatchEntry::CommitMark {
            xid: xid.to_owned(),
        });
    }
}

// call rust fn in C
#[doc(hidden)]
pub mod c {
    use super::*;

    #[no_mangle]
    pub unsafe extern "C" fn rust_write_batch_handler_put_cf(h: *mut (), column_family_id: u32, key: &&[u8], value: &&[u8]) {
        assert!(!h.is_null());
        let handler = h as *mut &mut WriteBatchHandler;
        (*handler).put_cf(column_family_id, key, value);
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_write_batch_handler_delete_cf(h: *mut (), column_family_id: u32, key: &&[u8]) {
        assert!(!h.is_null());
        let handler = h as *mut &mut WriteBatchHandler;
        (*handler).delete_cf(column_family_id, key);
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_write_batch_handler_single_delete_cf(h: *mut (), column_family_id: u32, key: &&[u8]) {
        assert!(!h.is_null());
        let handler = h as *mut &mut WriteBatchHandler;
        (*handler).single_delete_cf(column_family_id, key);
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_write_batch_handler_delete_range_cf(h: *mut (), column_family_id: u32,
                                                                      begin_key: &&[u8], end_key: &&[u8]) {
        assert!(!h.is_null());
        let handler = h as *mut &mut WriteBatchHandler;
        (*handler).delete_range_cf(column_family_id, begin_key, end_key);
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_write_batch_handler_merge_cf(h: *mut (), column_family_id: u32, key: &&[u8], value: &&[u8]) {
        assert!(!h.is_null());
        let handler = h as *mut &mut WriteBatchHandler;
        (*handler).merge_cf(column_family_id, key, value);
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_write_batch_handler_log_data(h: *mut (), blob: &&[u8]) {
        assert!(!h.is_null());
        let handler = h as *mut &mut WriteBatchHandler;
        (*handler).log_data(blob);
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_write_batch_handler_mark_begin_prepare(h: *mut ()) {
        assert!(!h.is_null());
        let handler = h as *mut &mut WriteBatchHandler;
        (*handler).mark_begin_prepare();
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_write_batch_handler_mark_end_prepare(h: *mut (), xid: &&[u8]) {
        assert!(!h.is_null());
        let handler = h as *mut &mut WriteBatchHandler;
        (*handler).mark_end_prepare(xid);
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_write_batch_handler_mark_rollback(h: *mut (), xid: &&[u8]) {
        assert!(!h.is_null());
        let handler = h as *mut &mut WriteBatchHandler;
        (*handler).mark_rollback(xid);
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_write_batch_handler_mark_commit(h: *mut (), xid: &&[u8]) {
        assert!(!h.is_null());
        let handler = h as *mut &mut WriteBatchHandler;
        (*handler).mark_commit(xid);
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_write_batch_handler_will_continue(h: *mut ()) -> c_uchar {
        assert!(!h.is_null());
        let handler = h as *mut &mut WriteBatchHandler;
        (*handler).will_continue() as c_uchar
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_write_batch_handler_drop(h: *mut ()) {
        assert!(!h.is_null());
        let handler = h as *mut &mut WriteBatchHandler;
        Box::from_raw(handler);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::rocksdb::*;

    #[test]
    fn write_batch_create() {
        let mut batch = WriteBatch::new();
        assert!(batch.count() == 0);
        batch.put(b"name", b"rocksdb");
        assert!(batch.count() == 1);
        batch.delete(b"name");
        assert_eq!(batch.count(), 2);
        assert!(format!("{:?}", batch).len() > 20);

        assert!(batch.has_put());
        assert!(batch.has_delete());
        assert!(!batch.has_commit());
        batch.put_log_data(b"Hello World!");

        let mut handler = WriteBatchIteratorHandler::default();
        let ret = batch.iterate(&mut handler);
        assert!(ret.is_ok(), "error: {:?}", ret);
        assert_eq!(handler.entries.len(), 3);
    }

    #[test]
    fn write_batch() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();

        let opt = Options::default().map_db_options(|db| db.create_if_missing(true));

        let db = DB::open(opt, &tmp_dir).unwrap();

        let mut batch = WriteBatch::new();
        batch
            .put(b"name", b"BY1CQ")
            .delete(b"name")
            .put(b"name", b"BH1XUW")
            .put(b"site", b"github");

        assert!(db.write(&WriteOptions::default(), batch).is_ok());

        assert_eq!(db.get(&ReadOptions::default(), b"name").unwrap().as_ref(), b"BH1XUW");
        assert_eq!(db.get(&ReadOptions::default(), b"site").unwrap().as_ref(), b"github");
    }
}
