#include "rocksdb/slice.h"

#include "rocks/ctypes.hpp"

using namespace rocksdb;

extern "C" {
rocks_pinnable_slice_t* rocks_pinnable_slice_create() {
  return new rocks_pinnable_slice_t;
}

void rocks_pinnable_slice_destroy(rocks_pinnable_slice_t* s) { delete s; }

const char* rocks_pinnable_slice_data(rocks_pinnable_slice_t* s) {
  return s->rep.data();
}

size_t rocks_pinnable_slice_size(rocks_pinnable_slice_t* s) {
  return s->rep.size();
}
}
