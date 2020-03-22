//! The metadata that describes a column family, a level, or a SST file,

use std::fmt;
use std::ops::Deref;

use crate::types::SequenceNumber;

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

impl fmt::Debug for ColumnFamilyMetaData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ColumnFamily")
            .field("name", &self.name)
            .field("file_count", &self.file_count)
            .field("size", &self.size)
            .finish()?;
        write!(f, "\n")?;
        for level in &self.levels {
            write!(f, "  > {:?}\n", level)?;
        }
        Ok(())
    }
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

impl fmt::Debug for LevelMetaData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Level")
            .field("level", &self.level)
            .field("files", &self.files)
            .finish()
    }
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

impl fmt::Debug for SstFileMetaData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SstFile").field("name", &self.name).finish()
    }
}

/// The full set of metadata associated with each SST file.
pub struct LiveFileMetaData {
    pub sst_file: SstFileMetaData,
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

impl fmt::Debug for LiveFileMetaData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LiveFile")
            .field("name", &self.name)
            .field("column_family_name", &self.column_family_name)
            .field("level", &self.level)
            .field("db_path", &self.db_path)
            .field("smallestkey", &String::from_utf8_lossy(&self.smallestkey))
            .field("largestkey", &String::from_utf8_lossy(&self.largestkey))
            .field("being_compacted", &self.being_compacted)
            .field("size", &self.size)
            .finish()
    }
}
