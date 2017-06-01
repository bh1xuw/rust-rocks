#include "rocksdb/perf_level.h"

#include "rocks/ctypes.hpp"

using namespace rocksdb;

//
extern "C" {
void rocks_set_perf_level(unsigned char level) {
  SetPerfLevel(static_cast<PerfLevel>(level));
}

unsigned char rocks_get_perf_level() {
  return static_cast<unsigned char>(GetPerfLevel());
}
}
