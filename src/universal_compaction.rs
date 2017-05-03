use std::u32;

/// Algorithm used to make a compaction request stop picking new files
/// into a single compaction run
#[repr(C)]
pub enum CompactionStopStyle {
    /// pick files of similar size
    SimilarSize,
    /// total size of picked files > next file
    TotalSize,
}


#[repr(C)]
pub struct CompactionOptionsUniversal {
    /// Percentage flexibilty while comparing file size. If the candidate file(s)
    /// size is 1% smaller than the next file's size, then include next file into
    /// this candidate set.
    /// Default: 1
    size_ratio: u32,

    /// The minimum number of files in a single compaction run. Default: 2
    min_merge_width: u32,

    /// The maximum number of files in a single compaction run. Default: UINT_MAX
    max_merge_width: u32,

    /// The size amplification is defined as the amount (in percentage) of
    /// additional storage needed to store a single byte of data in the database.
    /// For example, a size amplification of 2% means that a database that
    /// contains 100 bytes of user-data may occupy upto 102 bytes of
    /// physical storage. By this definition, a fully compacted database has
    /// a size amplification of 0%. Rocksdb uses the following heuristic
    /// to calculate size amplification: it assumes that all files excluding
    /// the earliest file contribute to the size amplification.
    /// Default: 200, which means that a 100 byte database could require upto
    /// 300 bytes of storage.
    max_size_amplification_percent: u32,

    /// If this option is set to be -1 (the default value), all the output files
    /// will follow compression type specified.
    ///
    /// If this option is not negative, we will try to make sure compressed
    /// size is just above this value. In normal cases, at least this percentage
    /// of data will be compressed.
    /// When we are compacting to a new file, here is the criteria whether
    /// it needs to be compressed: assuming here are the list of files sorted
    /// by generation time:
    ///    A1...An B1...Bm C1...Ct
    /// where A1 is the newest and Ct is the oldest, and we are going to compact
    /// B1...Bm, we calculate the total size of all the files as total_size, as
    /// well as  the total size of C1...Ct as total_C, the compaction output file
    /// will be compressed iff
    ///   total_C / total_size < this percentage
    /// Default: -1
    compression_size_percent: i32,

    /// The algorithm used to stop picking files into a single compaction run
    /// Default: kCompactionStopStyleTotalSize
    stop_style: CompactionStopStyle,

    /// Option to optimize the universal multi level compaction by enabling
    /// trivial move for non overlapping files.
    /// Default: false
    allow_trivial_move: bool,
}

impl Default for CompactionOptionsUniversal {
    fn default() -> Self {
        CompactionOptionsUniversal {
            size_ratio: 1,
            min_merge_width: 2,
            max_merge_width: u32::MAX,
            max_size_amplification_percent: 200,
            compression_size_percent: -1,
            stop_style: CompactionStopStyle::TotalSize,
            allow_trivial_move: false,
        }
    }
}
