//! Advanced Options

use std::os::raw::c_int;

use rocks_sys as ll;

use to_raw::ToRaw;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

/// In Level-based comapction, it Determines which file from a level to be
/// picked to merge to the next level. We suggest people try
/// kMinOverlappingRatio first when you tune your database.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
    raw: *mut ll::rocks_fifo_compaction_options_t,
}

impl ToRaw<ll::rocks_fifo_compaction_options_t> for CompactionOptionsFIFO {
    fn raw(&self) -> *mut ll::rocks_fifo_compaction_options_t {
        self.raw
    }
}

impl Default for CompactionOptionsFIFO {
    fn default() -> Self {
        CompactionOptionsFIFO { raw: unsafe { ll::rocks_fifo_compaction_options_create() } }
    }
}

impl Drop for CompactionOptionsFIFO {
    fn drop(&mut self) {
        unsafe { ll::rocks_fifo_compaction_options_destroy(self.raw) }
    }
}

impl CompactionOptionsFIFO {
    /// once the total sum of table files reaches this, we will delete the oldest
    /// table file
    ///
    /// Default: 1GB
    pub fn max_table_files_size(self, val: u64) -> Self {
        unsafe {
            ll::rocks_fifo_compaction_options_set_max_table_files_size(self.raw, val);
        }
        self
    }

    /// Drop files older than TTL. TTL based deletion will take precedence over
    /// size based deletion if ttl > 0.
    /// delete if `sst_file_creation_time < (current_time - ttl)`
    ///
    /// unit: seconds. Ex: 1 day = 1 * 24 * 60 * 60
    ///
    /// Default: 0 (disabled)
    pub fn ttl(self, val: u64) -> Self {
        unsafe {
            ll::rocks_fifo_compaction_options_set_ttl(self.raw, val);
        }
        self
    }

    /// If true, try to do compaction to compact smaller files into larger ones.
    /// Minimum files to compact follows options.level0_file_num_compaction_trigger
    /// and compaction won't trigger if average compact bytes per del file is
    /// larger than options.write_buffer_size. This is to protect large files
    /// from being compacted again.
    ///
    /// Default: false
    pub fn allow_compaction(self, val: bool) -> Self {
        unsafe {
            ll::rocks_fifo_compaction_options_set_allow_compaction(self.raw, val as u8);
        }
        self
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum UpdateStatus {
    /// Nothing to update
    Failed = 0,
    /// Value updated inplace
    Inplace = 1,
    /// No inplace update. Merged value set
    Updated = 2,
}


// FIXME: impled in ColumnFamilyOptions
// pub struct AdvancedColumnFamilyOptions {}
