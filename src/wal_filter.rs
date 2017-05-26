//! WALFilter allows an application to inspect write-ahead-log (WAL)
//! records or modify their processing on recovery.

use std::collections::BTreeMap;
use std::os::raw::c_int;

use write_batch::WriteBatch;

#[derive(Debug, Clone)]
pub enum WalProcessingOption {
    /// Continue processing as usual
    ContinueProcessing,
    /// Continue processing and change write batch
    ContinueAndChangeBatch(WriteBatch),
    /// Ignore the current record but continue processing of log(s)
    IgnoreCurrentRecord,
    /// Stop replay of logs and discard logs.
    /// Logs won't be replayed on subsequent recovery
    StopReplay,
    /// Corrupted record detected by filter
    CorruptedRecord,
}

impl WalProcessingOption {
    fn to_c(&self) -> c_int {
        use self::WalProcessingOption::*;

        match *self {
            ContinueProcessing |
            ContinueAndChangeBatch(_) => 0,
            IgnoreCurrentRecord => 1,
            StopReplay => 2,
            CorruptedRecord => 3,
        }
    }
}

/// WALFilter allows an application to inspect write-ahead-log (WAL)
/// records or modify their processing on recovery.
pub trait WalFilter {
    /// Provide `ColumnFamily->LogNumber` map to filter
    ///
    /// so that filter can determine whether a log number applies to a given
    /// column family (i.e. that log hasn't been flushed to SST already for the
    /// column family).
    ///
    /// We also pass in name->id map as only name is known during
    /// recovery (as handles are opened post-recovery).
    /// while write batch callbacks happen in terms of column family id.
    ///
    /// # Arguments
    ///
    /// * cf_lognumber_map - column_family_id to lognumber map
    /// * cf_name_id_map -   column_family_name to column_family_id map
    fn column_family_log_number_map(&mut self,
                                    cf_lognumber_map: &BTreeMap<u32, u64>,
                                    cf_name_id_map: &BTreeMap<String, u32>) {
    }

    /// LogRecord is invoked for each log record encountered for all the logs
    /// during replay on logs on recovery. This method can be used to:
    ///
    /// * inspect the record (using the batch parameter)
    /// * ignoring current record
    ///   (by returning WalProcessingOption::kIgnoreCurrentRecord)
    /// * reporting corrupted record
    ///   (by returning WalProcessingOption::kCorruptedRecord)
    /// * stop log replay
    ///   (by returning kStop replay) - please note that this implies
    ///   discarding the logs from current record onwards.
    ///
    /// # Arguments
    ///
    /// * log_number - log_number of the current log.
    ///
    ///   Filter might use this to determine if the log
    ///   record is applicable to a certain column family.
    /// * log_file_name - log file name - only for informational purposes
    /// * batch - batch encountered in the log during recovery
    /// * new_batch- new_batch to populate if filter wants to change
    ///   the batch (for example to filter some records out,
    ///   or alter some records).
    ///
    ///   Please note that the new batch MUST NOT contain
    ///   more records than original, else recovery would
    ///   be failed.
    /// * batch_changed -  Whether batch was changed by the filter.
    ///   It must be set to true if new_batch was populated,
    ///   else new_batch has no effect.
    ///
    /// Returns Processing option for the current record.
    ///
    /// Please see `WalProcessingOption` enum above for
    /// details.
    fn log_record_found(&self, log_number: u64, log_file_name: &str, batch: &WriteBatch) -> WalProcessingOption {
        WalProcessingOption::ContinueProcessing
    }

    /// Returns a name that identifies this WAL filter.
    ///
    /// The name will be printed to LOG file on start up for diagnosis.
    fn name(&self) -> &'static str {
        "RustWalFilter\0"
    }
}
