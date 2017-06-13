use std::fmt;

use types::SequenceNumber;

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
    Alive = 1
}


pub struct LogFile {
    pub path_name: String,
    pub log_number: u64,
    pub file_type: WalFileType,
    pub start_sequence: SequenceNumber,
    pub size_in_bytes: u64,
}

impl fmt::Debug for LogFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WalFile({:?}, {:?}, #{}, {} bytes)",
               self.path_name,
               self.file_type,
               self.log_number,
               self.size_in_bytes)
    }
}

