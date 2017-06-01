#include "rocksdb/perf_context.h"

#include "rocks/ctypes.hpp"

#include "rust_export.h"

using namespace rocksdb;

extern "C" {
rocks_perf_context_t* rocks_get_perf_context() {
  return reinterpret_cast<rocks_perf_context_t*>(&perf_context);
}

void rocks_perf_context_reset(rocks_perf_context_t* ctx) {
  reinterpret_cast<PerfContext*>(ctx)->Reset();
}

void rocks_perf_context_to_string(const rocks_perf_context_t* ctx,
                                  unsigned char exclude_zero_counters,
                                  void* s) {  // *mut String
  auto str = reinterpret_cast<const PerfContext*>(ctx)->ToString(
      exclude_zero_counters);
  rust_string_assign(s, str.data(), str.size());
}
}
