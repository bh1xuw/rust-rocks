//! `CompactionJobStats` used by event listener callback.
use rocks_sys as ll;

use std::fmt;
use std::slice;

use to_raw::FromRaw;

pub const MAX_PREFIX_LENGTH: usize = 8;


#[repr(C)]
pub struct CompactionJobStats {
    raw: *mut ll::rocks_compaction_job_stats_t,
}

impl FromRaw<ll::rocks_compaction_job_stats_t> for CompactionJobStats {
    unsafe fn from_ll(raw: *mut ll::rocks_compaction_job_stats_t) -> Self {
        CompactionJobStats { raw: raw }
    }
}

impl fmt::Debug for CompactionJobStats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CompactionJobStats")
            .field("elapsed_micros", &self.elapsed_micros())
            .field("num_input_records", &self.num_input_records())
            .field("num_input_files", &self.num_input_files())
            .field("num_input_files_at_output_level", &self.num_input_files_at_output_level())
            .field("num_output_records", &self.num_output_records())
            .field("num_output_files", &self.num_output_files())
            .field("is_manual_compaction", &self.is_manual_compaction())
            .field("total_input_bytes", &self.total_input_bytes())
            .field("total_output_bytes", &self.total_output_bytes())
            .field("num_records_replaced", &self.num_records_replaced())
            .field("total_input_raw_key_bytes", &self.total_input_raw_key_bytes())
            .field("total_input_raw_value_bytes", &self.total_input_raw_value_bytes())
            .field("num_input_deletion_records", &self.num_input_deletion_records())
            .field("num_expired_deletion_records", &self.num_expired_deletion_records())
            .field("num_corrupt_keys", &self.num_corrupt_keys())
            .finish()
    }
}

impl CompactionJobStats {
    /// the elapsed time in micro of this compaction.
    pub fn elapsed_micros(&self) -> u64 {
        unsafe { ll::rocks_compaction_job_stats_get_elapsed_micros(self.raw) }
    }

    /// the number of compaction input records.
    pub fn num_input_records(&self) -> u64 {
        unsafe { ll::rocks_compaction_job_stats_get_num_input_records(self.raw) }
    }

    /// the number of compaction input files.
    pub fn num_input_files(&self) -> usize {
        unsafe { ll::rocks_compaction_job_stats_get_num_input_files(self.raw) }
    }

    /// the number of compaction input files at the output level.
    pub fn num_input_files_at_output_level(&self) -> usize {
        unsafe { ll::rocks_compaction_job_stats_get_num_input_files_at_output_level(self.raw) }
    }

    /// the number of compaction output records.
    pub fn num_output_records(&self) -> u64 {
        unsafe { ll::rocks_compaction_job_stats_get_num_output_records(self.raw) }
    }

    /// the number of compaction output files.
    pub fn num_output_files(&self) -> usize {
        unsafe { ll::rocks_compaction_job_stats_get_num_output_files(self.raw) }
    }

    /// true if the compaction is a manual compaction
    pub fn is_manual_compaction(&self) -> bool {
        unsafe { ll::rocks_compaction_job_stats_get_is_manual_compaction(self.raw) != 0 }
    }

    /// the size of the compaction input in bytes.
    pub fn total_input_bytes(&self) -> u64 {
        unsafe { ll::rocks_compaction_job_stats_get_total_input_bytes(self.raw) }
    }

    /// the size of the compaction output in bytes.
    pub fn total_output_bytes(&self) -> u64 {
        unsafe { ll::rocks_compaction_job_stats_get_total_output_bytes(self.raw) }
    }

    /// number of records being replaced by newer record associated with same key.
    /// this could be a new value or a deletion entry for that key so this field
    /// sums up all updated and deleted keys
    pub fn num_records_replaced(&self) -> u64 {
        unsafe { ll::rocks_compaction_job_stats_get_num_records_replaced(self.raw) }
    }

    /// the sum of the uncompressed input keys in bytes.
    pub fn total_input_raw_key_bytes(&self) -> u64 {
        unsafe { ll::rocks_compaction_job_stats_get_total_input_raw_key_bytes(self.raw) }
    }

    /// the sum of the uncompressed input values in bytes.
    pub fn total_input_raw_value_bytes(&self) -> u64 {
        unsafe { ll::rocks_compaction_job_stats_get_total_input_raw_value_bytes(self.raw) }
    }

    /// the number of deletion entries before compaction. Deletion entries
    /// can disappear after compaction because they expired
    pub fn num_input_deletion_records(&self) -> u64 {
        unsafe { ll::rocks_compaction_job_stats_get_num_input_deletion_records(self.raw) }
    }

    /// number of deletion records that were found obsolete and discarded
    /// because it is not possible to delete any more keys with this entry
    /// (i.e. all possible deletions resulting from it have been completed)
    pub fn num_expired_deletion_records(&self) -> u64 {
        unsafe { ll::rocks_compaction_job_stats_get_num_expired_deletion_records(self.raw) }
    }

    /// number of corrupt keys (ParseInternalKey returned false when applied to
    /// the key) encountered and written out.
    pub fn num_corrupt_keys(&self) -> u64 {
        unsafe { ll::rocks_compaction_job_stats_get_num_corrupt_keys(self.raw) }
    }

    /// Following counters are only populated if
    /// options.report_bg_io_stats = true;

    /// Time spent on file's Append() call.
    pub fn file_write_nanos(&self) -> u64 {
        unsafe { ll::rocks_compaction_job_stats_get_file_write_nanos(self.raw) }
    }
    /// Time spent on sync file range.
    pub fn file_range_sync_nanos(&self) -> u64 {
        unsafe { ll::rocks_compaction_job_stats_get_file_range_sync_nanos(self.raw) }
    }

    /// Time spent on file fsync.
    pub fn file_fsync_nanos(&self) -> u64 {
        unsafe { ll::rocks_compaction_job_stats_get_file_fsync_nanos(self.raw) }
    }

    /// Time spent on preparing file write (falocate, etc)
    pub fn file_prepare_write_nanos(&self) -> u64 {
        unsafe { ll::rocks_compaction_job_stats_get_file_prepare_write_nanos(self.raw) }
    }

    /// 0-terminated strings storing the first 8 bytes of the smallest and
    /// largest key in the output.
    pub fn smallest_output_key_prefix(&self) -> &[u8] {
        let mut len = 0;
        unsafe {
            let ptr = ll::rocks_compaction_job_stats_get_smallest_output_key_prefix(self.raw, &mut len);
            slice::from_raw_parts(ptr as *const u8, len)
        }
    }
    pub fn largest_output_key_prefix(&self) -> &[u8] {
        let mut len = 0;
        unsafe {
            let ptr = ll::rocks_compaction_job_stats_get_largest_output_key_prefix(self.raw, &mut len);
            slice::from_raw_parts(ptr as *const u8, len)
        }
    }

    /// number of single-deletes which do not meet a put
    pub fn num_single_del_fallthru(&self) -> u64 {
        unsafe { ll::rocks_compaction_job_stats_get_num_single_del_fallthru(self.raw) }
    }

    /// number of single-deletes which meet something other than a put
    pub fn num_single_del_mismatch(&self) -> u64 {
        unsafe { ll::rocks_compaction_job_stats_get_num_single_del_mismatch(self.raw) }
    }
}
