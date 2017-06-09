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
}
