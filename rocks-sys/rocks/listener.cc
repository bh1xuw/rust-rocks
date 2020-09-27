#include "rocks/ctypes.hpp"

using namespace ROCKSDB_NAMESPACE;

extern "C" {

const char* rocks_flush_job_info_get_cf_name(const FlushJobInfo* info, size_t* len) {
  *len = info->cf_name.size();
  return info->cf_name.data();
}

const char* rocks_flush_job_info_get_file_path(const FlushJobInfo* info, size_t* len) {
  *len = info->file_path.size();
  return info->file_path.data();
}

uint64_t rocks_flush_job_info_get_thread_id(const FlushJobInfo* info) { return info->thread_id; }

uint64_t rocks_flush_job_info_get_job_id(const FlushJobInfo* info) { return info->job_id; }

unsigned char rocks_flush_job_info_get_triggered_writes_slowdown(const FlushJobInfo* info) {
  return info->triggered_writes_slowdown;
}

unsigned char rocks_flush_job_info_get_triggered_writes_stop(const FlushJobInfo* info) {
  return info->triggered_writes_stop;
}

uint64_t rocks_flush_job_info_get_smallest_seqno(const FlushJobInfo* info) { return info->smallest_seqno; }

uint64_t rocks_flush_job_info_get_largest_seqno(const FlushJobInfo* info) { return info->largest_seqno; }

rocks_table_props_t* rocks_flush_job_info_get_table_properties(const FlushJobInfo* info) {
  // deleter does nothing, this is a borrowed pointer.
  // since rocks_table_props_t use non-const pointer, const_cast here.
  return new rocks_table_props_t{
      std::shared_ptr<TableProperties>(const_cast<TableProperties*>(&info->table_properties), [](TableProperties*) {})};
}

// for TableFileDeletionInfo

const char* rocks_table_file_deletion_info_get_db_name(const TableFileDeletionInfo* info, size_t* len) {
  *len = info->db_name.size();
  return info->db_name.data();
}

const char* rocks_table_file_deletion_info_get_file_path(const TableFileDeletionInfo* info, size_t* len) {
  *len = info->file_path.size();
  return info->file_path.data();
}

uint64_t rocks_table_file_deletion_info_get_job_id(const TableFileDeletionInfo* info) { return info->job_id; }

void rocks_table_file_deletion_info_get_status(const TableFileDeletionInfo* info, rocks_status_t** status) {
  SaveError(status, Status(info->status));
}

// for CompactionJobInfo
const char* rocks_compaction_job_info_get_cf_name(const CompactionJobInfo* info, size_t* len) {
  *len = info->cf_name.size();
  return info->cf_name.data();
}

void rocks_compaction_job_info_get_status(const CompactionJobInfo* info, rocks_status_t** status) {
  SaveError(status, Status(info->status));
}

uint64_t rocks_compaction_job_info_get_thread_id(const CompactionJobInfo* info) { return info->thread_id; }

int rocks_compaction_job_info_get_job_id(const CompactionJobInfo* info) { return info->job_id; }

int rocks_compaction_job_info_get_base_input_level(const CompactionJobInfo* info) { return info->base_input_level; }

int rocks_compaction_job_info_get_output_level(const CompactionJobInfo* info) { return info->output_level; }

size_t rocks_compaction_job_info_get_input_files_num(const CompactionJobInfo* info) { return info->input_files.size(); }

// requires: files, sizes buf allocated with size acquired via above method
void rocks_compaction_job_info_get_input_files(const CompactionJobInfo* info, const char** files, size_t* sizes) {
  for (auto& f : info->input_files) {
    *(files++) = f.data();
    *(sizes++) = f.size();
  }
}

size_t rocks_compaction_job_info_get_output_files_num(const CompactionJobInfo* info) {
  return info->output_files.size();
}

// requires: files, sizes buf allocated with size acquired via above method
void rocks_compaction_job_info_get_output_files(const CompactionJobInfo* info, const char** files, size_t* sizes) {
  for (auto& f : info->output_files) {
    *(files++) = f.data();
    *(sizes++) = f.size();
  }
}

rocks_table_props_collection_t* rocks_compaction_job_info_get_table_properties(const CompactionJobInfo* info) {
  // FIXME: big map copy here?
  return new rocks_table_props_collection_t{info->table_properties};
}

int rocks_compaction_job_info_get_compaction_reason(const CompactionJobInfo* info) {
  return static_cast<int>(info->compaction_reason);
}

int rocks_compaction_job_info_get_compression(const CompactionJobInfo* info) {
  return static_cast<int>(info->compression);
}

const CompactionJobStats* rocks_compaction_job_info_get_stats(const CompactionJobInfo* info) { return &info->stats; }

// TableFileCreationInfo

typedef TableFileCreationInfo rocks_table_file_creation_info_t;
typedef TableFileCreationBriefInfo rocks_table_file_creation_brief_info_t;

uint64_t rocks_table_file_creation_info_get_file_size(const rocks_table_file_creation_info_t* info) {
  return info->file_size;
}

rocks_table_props_t* rocks_table_file_creation_info_get_table_properties(const rocks_table_file_creation_info_t* info) {
  return new rocks_table_props_t{
      std::shared_ptr<TableProperties>(const_cast<TableProperties*>(&info->table_properties), [](TableProperties*) {})};
}

void rocks_table_file_creation_info_get_status(const rocks_table_file_creation_info_t* info, rocks_status_t** status) {
  SaveError(status, Status(info->status));
}

// ** for ops::Deref + mem::transmute
const rocks_table_file_creation_brief_info_t* rocks_table_file_creation_info_get_brief_info(
    const rocks_table_file_creation_info_t* info) {
  return info;
}

// TableFileCreationBriefInfo
const char* rocks_table_file_creation_brief_info_get_db_name(const rocks_table_file_creation_brief_info_t* info,
                                                             size_t* len) {
  *len = info->db_name.size();
  return info->db_name.data();
}

const char* rocks_table_file_creation_brief_info_get_cf_name(const rocks_table_file_creation_brief_info_t* info,
                                                             size_t* len) {
  *len = info->cf_name.size();
  return info->cf_name.data();
}

const char* rocks_table_file_creation_brief_info_get_file_path(const rocks_table_file_creation_brief_info_t* info,
                                                               size_t* len) {
  *len = info->file_path.size();
  return info->file_path.data();
}

int rocks_table_file_creation_brief_info_get_job_id(const rocks_table_file_creation_brief_info_t* info) {
  return info->job_id;
}

int rocks_table_file_creation_brief_info_get_reason(const rocks_table_file_creation_brief_info_t* info) {
  return static_cast<int>(info->reason);
}

// MemTableInfo
typedef MemTableInfo rocks_mem_table_info_t;

const char* rocks_mem_table_info_get_cf_name(const rocks_mem_table_info_t* info, size_t* len) {
  *len = info->cf_name.size();
  return info->cf_name.data();
}
uint64_t rocks_mem_table_info_get_first_seqno(const rocks_mem_table_info_t* info) { return info->first_seqno; }
uint64_t rocks_mem_table_info_get_earliest_seqno(const rocks_mem_table_info_t* info) { return info->earliest_seqno; }
uint64_t rocks_mem_table_info_get_num_entries(const rocks_mem_table_info_t* info) { return info->num_entries; }
uint64_t rocks_mem_table_info_get_num_deletes(const rocks_mem_table_info_t* info) { return info->num_deletes; }

// ExternalFileIngestionInfo
typedef ExternalFileIngestionInfo rocks_external_file_ingestion_info_t;

const char* rocks_external_file_ingestion_info_get_cf_name(const rocks_external_file_ingestion_info_t* info,
                                                           size_t* len) {
  *len = info->cf_name.size();
  return info->cf_name.data();
}

const char* rocks_external_file_ingestion_info_get_external_file_path(const rocks_external_file_ingestion_info_t* info,
                                                                      size_t* len) {
  *len = info->external_file_path.size();
  return info->external_file_path.data();
}

const char* rocks_external_file_ingestion_info_get_internal_file_path(const rocks_external_file_ingestion_info_t* info,
                                                                      size_t* len) {
  *len = info->internal_file_path.size();
  return info->internal_file_path.data();
}

uint64_t rocks_external_file_ingestion_info_get_global_seqno(const rocks_external_file_ingestion_info_t* info) {
  return info->global_seqno;
}

rocks_table_props_t* rocks_external_file_ingestion_info_get_table_properties(
    const rocks_external_file_ingestion_info_t* info) {
  return new rocks_table_props_t{
      std::shared_ptr<TableProperties>(const_cast<TableProperties*>(&info->table_properties), [](TableProperties*) {})};
}
}
