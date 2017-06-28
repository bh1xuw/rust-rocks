

pub const MAX_PREFIX_LENGTH: usize = 8;


pub struct CompactionJobStats<'a> {
    /// the elapsed time in micro of this compaction.
    pub elapsed_micros: u64,

    /// the number of compaction input records.
    pub num_input_records: u64,
    /// the number of compaction input files.
    pub num_input_files: usize,
    /// the number of compaction input files at the output level.
    pub num_input_files_at_output_level: usize,

    /// the number of compaction output records.
    pub num_output_records: u64,
    /// the number of compaction output files.
    pub num_output_files: usize,

    /// true if the compaction is a manual compaction
    pub is_manual_compaction: bool,

    /// the size of the compaction input in bytes.
    pub total_input_bytes: u64,
    /// the size of the compaction output in bytes.
    pub total_output_bytes: u64,

    /// number of records being replaced by newer record associated with same key.
    /// this could be a new value or a deletion entry for that key so this field
    /// sums up all updated and deleted keys
    pub num_records_replaced: u64,

    /// the sum of the uncompressed input keys in bytes.
    pub total_input_raw_key_bytes: u64,
    /// the sum of the uncompressed input values in bytes.
    pub total_input_raw_value_bytes: u64,

    /// the number of deletion entries before compaction. Deletion entries
    /// can disappear after compaction because they expired
    pub num_input_deletion_records: u64,
    /// number of deletion records that were found obsolete and discarded
    /// because it is not possible to delete any more keys with this entry
    /// (i.e. all possible deletions resulting from it have been completed)
    pub num_expired_deletion_records: u64,

    /// number of corrupt keys (ParseInternalKey returned false when applied to
    /// the key) encountered and written out.
    pub num_corrupt_keys: u64,

    /// Following counters are only populated if
    /// options.report_bg_io_stats = true;

    /// Time spent on file's Append() call.
    pub file_write_nanos: u64,

    /// Time spent on sync file range.
    pub file_range_sync_nanos: u64,

    /// Time spent on file fsync.
    pub file_fsync_nanos: u64,

    /// Time spent on preparing file write (falocate, etc)
    pub file_prepare_write_nanos: u64,

    /// 0-terminated strings storing the first 8 bytes of the smallest and
    /// largest key in the output.
    pub smallest_output_key_prefix: &'a [u8],
    pub largest_output_key_prefix: &'a [u8],

    /// number of single-deletes which do not meet a put
    pub num_single_del_fallthru: u64,

    /// number of single-deletes which meet something other than a put
    pub num_single_del_mismatch: u64,
}
