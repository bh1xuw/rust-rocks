#include "rocksdb/convenience.h"

#include "rocks/ctypes.hpp"

using namespace rocksdb;

extern "C" {
int* rocks_get_supported_compressions(size_t* len) {
  auto types = rocksdb::GetSupportedCompressions();
  *len = types.size();
  int* ptr = new int[*len];
  for (auto i = 0; i < *len; i++) {
    ptr[i] = static_cast<int>(types[i]);
  }
  return ptr;
}

void rocks_get_supported_compressions_destroy(int* ptr) { delete[] ptr; }

void rocks_cancel_all_background_work(rocks_db_t* db, unsigned char wait) {
  CancelAllBackgroundWork(db->rep, wait != 0);
}

void rocks_db_delete_files_in_range(rocks_db_t* db, rocks_column_family_handle_t* column_family, const char* begin_ptr,
                                    size_t begin_len, const char* end_ptr, size_t end_len, rocks_status_t** status) {
  auto begin = Slice(begin_ptr, begin_len);
  auto end = Slice(end_ptr, end_len);
  auto st = DeleteFilesInRange(db->rep, column_family->rep, &begin, &end);
  SaveError(status, std::move(st));
}

cxx_string_t* rocks_get_string_from_dboptions(rocks_dboptions_t* opts) {
  auto str = new std::string();
  auto st = GetStringFromDBOptions(str, opts->rep);
  if (st.ok()) {
    return reinterpret_cast<cxx_string_t*>(str);
  } else {  // from RocksDB's code, seems never fails
    delete str;
    return nullptr;
  }
}

cxx_string_t* rocks_get_string_from_cfoptions(rocks_cfoptions_t* opts) {
  auto str = new std::string();
  auto st = GetStringFromColumnFamilyOptions(str, opts->rep);
  if (st.ok()) {
    return reinterpret_cast<cxx_string_t*>(str);
  } else {  // from RocksDB's code, seems never fails
    delete str;
    return nullptr;
  }
}
}
