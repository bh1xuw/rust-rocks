#include "rocksdb/sst_file_writer.h"
#include "rocksdb/comparator.h"

#include "rocks/ctypes.hpp"

using namespace rocksdb;

using std::shared_ptr;

extern "C" {
rocks_external_sst_file_info_t* rocks_external_sst_file_info_create() { return new rocks_external_sst_file_info_t; }

void rocks_external_sst_file_info_destroy(rocks_external_sst_file_info_t* info) { delete info; }

const char* rocks_external_sst_file_info_get_file_path(rocks_external_sst_file_info_t* info, size_t* len) {
  *len = info->rep.file_path.size();
  return info->rep.file_path.data();
}

const char* rocks_external_sst_file_info_get_smallest_key(rocks_external_sst_file_info_t* info, size_t* len) {
  *len = info->rep.smallest_key.size();
  return info->rep.smallest_key.data();
}

const char* rocks_external_sst_file_info_get_largest_key(rocks_external_sst_file_info_t* info, size_t* len) {
  *len = info->rep.largest_key.size();
  return info->rep.largest_key.data();
}

uint64_t rocks_external_sst_file_info_get_sequence_number(rocks_external_sst_file_info_t* info) {
  return info->rep.sequence_number;
}

uint64_t rocks_external_sst_file_info_get_file_size(rocks_external_sst_file_info_t* info) {
  return info->rep.file_size;
}

uint64_t rocks_external_sst_file_info_get_num_entries(rocks_external_sst_file_info_t* info) {
  return info->rep.num_entries;
}

int32_t rocks_external_sst_file_info_get_version(rocks_external_sst_file_info_t* info) { return info->rep.version; }
}

extern "C" {
rocks_sst_file_writer_t* rocks_sst_file_writer_create_from_c_comparator(const rocks_envoptions_t* env_options,
                                                                        const rocks_options_t* options,
                                                                        const Comparator* comparator,
                                                                        rocks_column_family_handle_t* column_family,
                                                                        unsigned char invalidate_page_cache) {
  rocks_sst_file_writer_t* result = new rocks_sst_file_writer_t;
  result->rep =
      new SstFileWriter(env_options->rep, options->rep, comparator,
                        (column_family != nullptr) ? column_family->rep : nullptr, invalidate_page_cache != 0);
  return result;
}

rocks_sst_file_writer_t* rocks_sst_file_writer_create_from_rust_comparator(const rocks_envoptions_t* env_options,
                                                                           const rocks_options_t* options,
                                                                           const void* comparator_trait_obj,
                                                                           rocks_column_family_handle_t* column_family,
                                                                           unsigned char invalidate_page_cache) {
  rocks_sst_file_writer_t* result = new rocks_sst_file_writer_t;
  result->rep = new SstFileWriter(
      env_options->rep, options->rep, new rocks_comparator_t{(void*)comparator_trait_obj},  // FIXME: memory leaks
      (column_family != nullptr) ? column_family->rep : nullptr, invalidate_page_cache != 0);
  return result;
}

void rocks_sst_file_writer_destroy(rocks_sst_file_writer_t* writer) {
  delete writer->rep;
  delete writer;
}

void rocks_sst_file_writer_open(rocks_sst_file_writer_t* writer, const char* file_path, const size_t file_path_len,
                                rocks_status_t** status) {
  auto path = std::string(file_path, file_path_len);
  SaveError(status, std::move(writer->rep->Open(path)));
}

void rocks_sst_file_writer_put(rocks_sst_file_writer_t* writer, const char* key, const size_t key_len,
                               const char* value, const size_t value_len, rocks_status_t** status) {
  auto st = writer->rep->Put(Slice(key, key_len), Slice(value, value_len));
  SaveError(status, std::move(st));
}

void rocks_sst_file_writer_merge(rocks_sst_file_writer_t* writer, const char* key, const size_t key_len,
                                 const char* value, const size_t value_len, rocks_status_t** status) {
  auto st = writer->rep->Merge(Slice(key, key_len), Slice(value, value_len));
  SaveError(status, std::move(st));
}

void rocks_sst_file_writer_delete(rocks_sst_file_writer_t* writer, const char* key, const size_t key_len,
                                  rocks_status_t** status) {
  auto st = writer->rep->Delete(Slice(key, key_len));
  SaveError(status, std::move(st));
}

void rocks_sst_file_writer_finish(rocks_sst_file_writer_t* writer, rocks_external_sst_file_info_t* info,
                                  rocks_status_t** status) {
  auto info_ptr = (info != nullptr) ? &info->rep : nullptr;
  SaveError(status, std::move(writer->rep->Finish(info_ptr)));
}

uint64_t rocks_sst_file_writer_file_size(rocks_sst_file_writer_t* writer) { return writer->rep->FileSize(); }
}
