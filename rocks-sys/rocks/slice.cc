#include "rocksdb/slice.h"

#include "rocks/ctypes.hpp"

using namespace rocksdb;

extern "C" {
rocks_pinnable_slice_t* rocks_pinnable_slice_create() {
  return new rocks_pinnable_slice_t;
}
}
