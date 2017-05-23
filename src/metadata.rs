use std::ops::Deref;

use rocks_sys as ll;

use status::Status;

use types::SequenceNumber;


/// The metadata that describes a column family.
pub struct ColumnFamilyMetaData {
    /// The size of this column family in bytes, which is equal to the sum of
    /// the file size of its "levels".
    pub size: u64,
    /// The number of files in this column family.
    pub file_count: usize,
    /// The name of the column family.
    pub name: String,
    /// The metadata of all levels in this column family.
    pub levels: Vec<LevelMetaData>,
}

/// The metadata that describes a level.
pub struct LevelMetaData {
    /// The level which this meta data describes.
    pub level: u32,
    /// The size of this level in bytes, which is equal to the sum of
    /// the file size of its "files".
    pub size: u64,
    /// The metadata of all sst files in this level.
    pub files: Vec<SstFileMetaData>,
}

/// The metadata that describes a SST file.
pub struct SstFileMetaData {
    /// File size in bytes.
    pub size: u64,
    /// The name of the file.
    pub name: String,
    /// The full path where the file locates.
    pub db_path: String,

    /// Smallest sequence number in file.
    pub smallest_seqno: SequenceNumber,
    /// Largest sequence number in file.
    pub largest_seqno: SequenceNumber,
    /// Smallest user defined key in the file.
    pub smallestkey: Vec<u8>,
    /// Largest user defined key in the file.
    pub largestkey: Vec<u8>,
    /// true if the file is currently being compacted.
    pub being_compacted: bool,
}

/// The full set of metadata associated with each SST file.
pub struct LiveFileMetaData {
    sst_file: SstFileMetaData,
    /// Name of the column family
    pub column_family_name: String,
    /// Level at which this file resides.
    pub level: u32,
}

impl Deref for LiveFileMetaData {
    type Target = SstFileMetaData;

    fn deref(&self) -> &SstFileMetaData {
        &self.sst_file
    }
}
