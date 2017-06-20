//! WAL logs

use std::fmt;

use types::SequenceNumber;

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
            self.path_name,
            self.file_type,
            self.log_number,
            self.size_in_bytes
        )
    }
}
