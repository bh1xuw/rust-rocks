#include "rocks/ctypes.hpp"

using namespace rocksdb;

extern "C" {

const char* rocks_flush_job_info_get_cf_name(const FlushJobInfo* info,
                                             size_t* len) {
  *len = info->cf_name.size();
  return info->cf_name.data();
}

const char* rocks_flush_job_info_get_file_path(const FlushJobInfo* info,
                                               size_t* len) {
  *len = info->file_path.size();
  return info->file_path.data();
}

uint64_t rocks_flush_job_info_get_thread_id(const FlushJobInfo* info) {
  return info->thread_id;
}

uint64_t rocks_flush_job_info_get_job_id(const FlushJobInfo* info) {
  return info->job_id;
}

unsigned char rocks_flush_job_info_get_triggered_writes_slowdown(
    const FlushJobInfo* info) {
  return info->triggered_writes_slowdown;
}

unsigned char rocks_flush_job_info_get_triggered_writes_stop(
    const FlushJobInfo* info) {
  return info->triggered_writes_stop;
}

uint64_t rocks_flush_job_info_get_smallest_seqno(const FlushJobInfo* info) {
  return info->smallest_seqno;
}

uint64_t rocks_flush_job_info_get_largest_seqno(const FlushJobInfo* info) {
  return info->largest_seqno;
}

rocks_table_props_t* rocks_flush_job_info_get_table_properties(
    const FlushJobInfo* info) {
  // deleter does nothing, this is a borrowed pointer.
  // since rocks_table_props_t use non-const pointer, const_cast here.
  return new rocks_table_props_t{std::shared_ptr<TableProperties>(
      const_cast<TableProperties*>(&info->table_properties),
      [](TableProperties*) {})};
}
}
