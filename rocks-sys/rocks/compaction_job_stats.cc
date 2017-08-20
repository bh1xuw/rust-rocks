#include "rocks/ctypes.hpp"

using namespace rocksdb;

typedef CompactionJobStats rocks_compaction_job_stats_t;

extern "C" {

uint64_t rocks_compaction_job_stats_get_elapsed_micros(const rocks_compaction_job_stats_t* stats) {
  return reinterpret_cast<const CompactionJobStats*>(stats)->elapsed_micros;
}
uint64_t rocks_compaction_job_stats_get_num_input_records(const rocks_compaction_job_stats_t* stats) {
  return reinterpret_cast<const CompactionJobStats*>(stats)->num_input_records;
}
size_t rocks_compaction_job_stats_get_num_input_files(const rocks_compaction_job_stats_t* stats) {
  return reinterpret_cast<const CompactionJobStats*>(stats)->num_input_files;
}
size_t rocks_compaction_job_stats_get_num_input_files_at_output_level(const rocks_compaction_job_stats_t* stats) {
  return reinterpret_cast<const CompactionJobStats*>(stats)->num_input_files_at_output_level;
}
uint64_t rocks_compaction_job_stats_get_num_output_records(const rocks_compaction_job_stats_t* stats) {
  return reinterpret_cast<const CompactionJobStats*>(stats)->num_output_records;
}
size_t rocks_compaction_job_stats_get_num_output_files(const rocks_compaction_job_stats_t* stats) {
  return reinterpret_cast<const CompactionJobStats*>(stats)->num_output_files;
}
unsigned char rocks_compaction_job_stats_get_is_manual_compaction(const rocks_compaction_job_stats_t* stats) {
  return reinterpret_cast<const CompactionJobStats*>(stats)->is_manual_compaction;
}
uint64_t rocks_compaction_job_stats_get_total_input_bytes(const rocks_compaction_job_stats_t* stats) {
  return reinterpret_cast<const CompactionJobStats*>(stats)->total_input_bytes;
}
uint64_t rocks_compaction_job_stats_get_total_output_bytes(const rocks_compaction_job_stats_t* stats) {
  return reinterpret_cast<const CompactionJobStats*>(stats)->total_output_bytes;
}
uint64_t rocks_compaction_job_stats_get_num_records_replaced(const rocks_compaction_job_stats_t* stats) {
  return reinterpret_cast<const CompactionJobStats*>(stats)->num_records_replaced;
}
uint64_t rocks_compaction_job_stats_get_total_input_raw_key_bytes(const rocks_compaction_job_stats_t* stats) {
  return reinterpret_cast<const CompactionJobStats*>(stats)->total_input_raw_key_bytes;
}
uint64_t rocks_compaction_job_stats_get_total_input_raw_value_bytes(const rocks_compaction_job_stats_t* stats) {
  return reinterpret_cast<const CompactionJobStats*>(stats)->total_input_raw_value_bytes;
}
uint64_t rocks_compaction_job_stats_get_num_input_deletion_records(const rocks_compaction_job_stats_t* stats) {
  return reinterpret_cast<const CompactionJobStats*>(stats)->num_input_deletion_records;
}
uint64_t rocks_compaction_job_stats_get_num_expired_deletion_records(const rocks_compaction_job_stats_t* stats) {
  return reinterpret_cast<const CompactionJobStats*>(stats)->num_expired_deletion_records;
}
uint64_t rocks_compaction_job_stats_get_num_corrupt_keys(const rocks_compaction_job_stats_t* stats) {
  return reinterpret_cast<const CompactionJobStats*>(stats)->num_corrupt_keys;
}
uint64_t rocks_compaction_job_stats_get_file_write_nanos(const rocks_compaction_job_stats_t* stats) {
  return reinterpret_cast<const CompactionJobStats*>(stats)->file_write_nanos;
}
uint64_t rocks_compaction_job_stats_get_file_range_sync_nanos(const rocks_compaction_job_stats_t* stats) {
  return reinterpret_cast<const CompactionJobStats*>(stats)->file_range_sync_nanos;
}
uint64_t rocks_compaction_job_stats_get_file_fsync_nanos(const rocks_compaction_job_stats_t* stats) {
  return reinterpret_cast<const CompactionJobStats*>(stats)->file_fsync_nanos;
}
uint64_t rocks_compaction_job_stats_get_file_prepare_write_nanos(const rocks_compaction_job_stats_t* stats) {
  return reinterpret_cast<const CompactionJobStats*>(stats)->file_prepare_write_nanos;
}

const char* rocks_compaction_job_stats_get_smallest_output_key_prefix(const rocks_compaction_job_stats_t* stats,
                                                                      size_t* len) {
  *len = reinterpret_cast<const CompactionJobStats*>(stats)->smallest_output_key_prefix.size();
  return reinterpret_cast<const CompactionJobStats*>(stats)->smallest_output_key_prefix.data();
}

const char* rocks_compaction_job_stats_get_largest_output_key_prefix(const rocks_compaction_job_stats_t* stats,
                                                                     size_t* len) {
  *len = reinterpret_cast<const CompactionJobStats*>(stats)->largest_output_key_prefix.size();
  return reinterpret_cast<const CompactionJobStats*>(stats)->largest_output_key_prefix.data();
}

uint64_t rocks_compaction_job_stats_get_num_single_del_fallthru(const rocks_compaction_job_stats_t* stats) {
  return reinterpret_cast<const CompactionJobStats*>(stats)->num_single_del_fallthru;
}
uint64_t rocks_compaction_job_stats_get_num_single_del_mismatch(const rocks_compaction_job_stats_t* stats) {
  return reinterpret_cast<const CompactionJobStats*>(stats)->num_single_del_mismatch;
}
}
