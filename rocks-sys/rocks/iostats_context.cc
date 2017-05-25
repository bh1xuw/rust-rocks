#include "rocksdb/iostats_context.h"

#include "rocks/ctypes.hpp"

#include "rust_export.h"

using namespace rocksdb;

extern "C" {
  rocks_iostats_context_t* rocks_get_iostats_context() {
    return reinterpret_cast<rocks_iostats_context_t*>(&iostats_context);
  }

  void rocks_iostats_context_reset(rocks_iostats_context_t* ctx) {
    reinterpret_cast<IOStatsContext*>(ctx)->Reset();
  }

  void rocks_iostats_context_to_string(const rocks_iostats_context_t* ctx,
                                       unsigned char exclude_zero_counters,
                                       void* s) { // *mut String
    auto str = reinterpret_cast<const IOStatsContext*>(ctx)->ToString(exclude_zero_counters);
    rust_string_assign(s, str.data(), str.size());
  }
}
