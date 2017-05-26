//! Advanced Options

use std::os::raw::{c_char, c_int};

#[repr(C)]
pub enum CompactionStyle {
    /// level based compaction style
    CompactionStyleLevel = 0x0,
    /// Universal compaction style
    /// Not supported in ROCKSDB_LITE.
    CompactionStyleUniversal = 0x1,
    /// FIFO compaction style
    /// Not supported in ROCKSDB_LITE
    CompactionStyleFIFO = 0x2,
    /// Disable background compaction. Compaction jobs are submitted
    /// via CompactFiles().
    /// Not supported in ROCKSDB_LITE
    CompactionStyleNone = 0x3,
}

// In Level-based comapction, it Determines which file from a level to be
// picked to merge to the next level. We suggest people try
// kMinOverlappingRatio first when you tune your database.
#[repr(C)]
pub enum CompactionPri {
    /// Slightly Priotize larger files by size compensated by #deletes
    ByCompensatedSize = 0x0,
    /// First compact files whose data's latest update time is oldest.
    /// Try this if you only update some hot keys in small ranges.
    OldestLargestSeqFirst = 0x1,
    /// First compact files whose range hasn't been compacted to the next level
    /// for the longest. If your updates are random across the key space,
    /// write amplification is slightly better with this option.
    OldestSmallestSeqFirst = 0x2,
    /// First compact files whose ratio between overlapping size in next level
    /// and its size is the smallest. It in many cases can optimize write
    /// amplification.
    MinOverlappingRatio = 0x3,
}

#[repr(C)]
pub struct CompactionOptionsFIFO {
    /// once the total sum of table files reaches this, we will delete the oldest
    /// table file
    /// Default: 1GB
    max_table_files_size: u64,
}

impl CompactionOptionsFIFO {
    pub fn new(max_table_files_size: u64) -> CompactionOptionsFIFO {
        CompactionOptionsFIFO { max_table_files_size: max_table_files_size }
    }
}

impl Default for CompactionOptionsFIFO {
    fn default() -> Self {
        CompactionOptionsFIFO::new(1 * 1024 * 1024 * 1024)
    }
}

/// Compression options for different compression algorithms like Zlib
#[repr(C)]
pub struct CompressionOptions {
    pub window_bits: c_int,
    pub level: c_int,
    pub strategy: c_int,
    /// Maximum size of dictionary used to prime the compression library. Currently
    /// this dictionary will be constructed by sampling the first output file in a
    /// subcompaction when the target level is bottommost. This dictionary will be
    /// loaded into the compression library before compressing/uncompressing each
    /// data block of subsequent files in the subcompaction. Effectively, this
    /// improves compression ratios when there are repetitions across data blocks.
    /// A value of 0 indicates the feature is disabled.
    /// Default: 0.
    pub max_dict_bytes: u32,
}

impl CompressionOptions {
    pub fn new(wbits: c_int, lev: c_int, strategy: c_int, max_dict_bytes: u32) -> CompressionOptions {
        CompressionOptions {
            window_bits: wbits,
            level: lev,
            strategy: strategy,
            max_dict_bytes: max_dict_bytes,
        }
    }
}

impl Default for CompressionOptions {
    fn default() -> Self {
        CompressionOptions::new(-14, -1, 0, 0)
    }
}

/// Return status For inplace update callback
#[repr(C)]
pub enum UpdateStatus {
    /// Nothing to update
    Failed = 0,
    /// Value updated inplace
    Inplace = 1,
    /// No inplace update. Merged value set
    Updated = 2,
}


// TODO: not in current version
pub struct AdvancedColumnFamilyOptions {}
