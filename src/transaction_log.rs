//! WAL logs

use std::fmt;
use std::iter::Iterator;
use std::ptr;

use rocks_sys as ll;

use crate::error::Status;
use crate::to_raw::{FromRaw, ToRaw};
use crate::types::SequenceNumber;
use crate::write_batch::WriteBatch;
use crate::Result;

/// Is WAL file archived or alive
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum WalFileType {
    /// Indicates that WAL file is in archive directory. WAL files are moved from
    /// the main db directory to archive directory once they are not live and stay
    /// there until cleaned up. Files are cleaned depending on archive size
    /// `Options::WAL_size_limit_MB` and time since last cleaning
    /// `Options::WAL_ttl_seconds`.
    Archived = 0,
    /// Indicates that WAL file is live and resides in the main db directory
    Alive = 1,
}

/// Represents a single WAL file
pub struct LogFile {
    /// Returns log file's pathname relative to the main db dir
    /// Eg. For a live-log-file = /000003.log
    ///     For an archived-log-file = /archive/000003.log
    pub path_name: String,
    /// Primary identifier for log file.
    /// This is directly proportional to creation time of the log file
    pub log_number: u64,
    /// Log file can be either alive or archived
    pub file_type: WalFileType,
    /// Starting sequence number of writebatch written in this log file
    pub start_sequence: SequenceNumber,
    /// Size of log file on disk in Bytes
    pub size_in_bytes: u64,
}

impl fmt::Debug for LogFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "WalFile({:?}, {:?}, #{}, {} bytes)",
            self.path_name, self.file_type, self.log_number, self.size_in_bytes
        )
    }
}

/// Single write batch result returned by `TransactionLogIterator`
#[derive(Debug)]
pub struct BatchResult {
    pub sequence: SequenceNumber,
    pub write_batch: WriteBatch,
}

/// A `TransactionLogIterator` is used to iterate over the transactions in a db.
/// One run of the iterator is continuous, i.e. the iterator will stop at the
/// beginning of any gap in sequences
#[derive(Debug)]
pub struct TransactionLogIterator {
    raw: *mut ll::rocks_transaction_log_iterator_t,
}

impl ToRaw<ll::rocks_transaction_log_iterator_t> for TransactionLogIterator {
    fn raw(&self) -> *mut ll::rocks_transaction_log_iterator_t {
        self.raw
    }
}

impl FromRaw<ll::rocks_transaction_log_iterator_t> for TransactionLogIterator {
    unsafe fn from_ll(raw: *mut ll::rocks_transaction_log_iterator_t) -> TransactionLogIterator {
        TransactionLogIterator { raw: raw }
    }
}

impl Drop for TransactionLogIterator {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_transaction_log_iterator_destory(self.raw);
        }
    }
}

impl TransactionLogIterator {
    /// An iterator is either positioned at a WriteBatch or not valid.
    /// This method returns true if the iterator is valid.
    /// Can read data from a valid iterator.
    pub fn is_valid(&self) -> bool {
        unsafe { ll::rocks_transaction_log_iterator_valid(self.raw) != 0 }
    }

    /// Moves the iterator to the next WriteBatch.
    ///
    /// REQUIRES: Valid() to be true.
    ///
    /// Rust: avoid name collision with `Iterator::next`
    pub fn move_next(&mut self) {
        unsafe {
            ll::rocks_transaction_log_iterator_next(self.raw);
        }
    }

    /// Returns ok if the iterator is valid.
    /// Returns the Error when something has gone wrong.
    pub fn status(&self) -> Result<()> {
        let mut status = ptr::null_mut();
        unsafe {
            ll::rocks_transaction_log_iterator_status(self.raw, &mut status);
            Status::from_ll(status)
        }
    }

    /// If valid return's the current write_batch and the sequence number of the
    /// earliest transaction contained in the batch.
    ///
    /// ONLY use if Valid() is true and status() is OK.
    pub fn get_batch(&self) -> BatchResult {
        let mut seq_no = 0;
        unsafe {
            let batch_raw_ptr = ll::rocks_transaction_log_iterator_get_batch(self.raw, &mut seq_no);
            BatchResult {
                sequence: SequenceNumber(seq_no),
                write_batch: WriteBatch::from_ll(batch_raw_ptr),
            }
        }
    }
}

impl Iterator for TransactionLogIterator {
    type Item = BatchResult;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_valid() && self.status().is_ok() {
            let batch = self.get_batch();
            self.move_next();
            Some(batch)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::rocksdb::*;

    use crate::write_batch::WriteBatchIteratorHandler;

    #[test]
    fn transaction_log_iter() {
        let tmp_dir = ::tempdir::TempDir::new_in("", "rocks").unwrap();
        let db = DB::open(
            Options::default()
                .map_db_options(|db| {
                    db.create_if_missing(true)
                        .wal_ttl_seconds(1000000)
                        .wal_size_limit_mb(1024)
                })
                .map_cf_options(|cf| cf.disable_auto_compactions(false)), // disable
            &tmp_dir,
        )
        .unwrap();

        for i in 0..100 {
            let key = format!("k{}", i);
            let val = format!("v{}", i * i);

            let mut batch = WriteBatch::default();
            batch
                .put(format!("K{}", i).as_bytes(), format!("V{}", i * i).as_bytes())
                .put(format!("M{}", i).as_bytes(), format!("V{}", i).as_bytes())
                .put(format!("N{}", i).as_bytes(), format!("V{}", i * i * i).as_bytes());

            assert!(db.write(WriteOptions::default_instance(), batch).is_ok());

            if i % 9 == 0 {
                assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
            }
        }

        let it = db.get_updates_since(2000.into());
        assert!(it.is_err());

        let it = db.get_updates_since(20.into());
        assert!(it.is_ok());

        let mut it = it.unwrap();
        assert!(it.is_valid());
        assert!(it.status().is_ok());
        assert!(it.next().is_some());
        let batch = it.get_batch();
        println!("batch => {:?}", batch);
        assert!(batch.sequence.0 >= 20);
        assert_eq!(batch.write_batch.count(), 3);

        let mut handler = WriteBatchIteratorHandler::default();
        let ret = batch.write_batch.iterate(&mut handler);
        assert!(ret.is_ok(), "error: {:?}", ret);
        assert_eq!(handler.entries.len(), 3);

        for batch in db.get_updates_since(20.into()).unwrap() {
            // first batch will contains current since seq_no, so jump backwards
            assert!(batch.sequence.0 > 20 - 3);
        }
    }
}
