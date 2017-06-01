#include "rocksdb/universal_compaction.h"

#include "rocks/ctypes.hpp"

using namespace rocksdb;

extern "C" {
rocks_universal_compaction_options_t*
rocks_universal_compaction_options_create() {
  return new rocks_universal_compaction_options_t;
}

void rocks_universal_compaction_options_set_size_ratio(
    rocks_universal_compaction_options_t* uco, int ratio) {
  uco->rep.size_ratio = ratio;
}

void rocks_universal_compaction_options_set_min_merge_width(
    rocks_universal_compaction_options_t* uco, int w) {
  uco->rep.min_merge_width = w;
}

void rocks_universal_compaction_options_set_max_merge_width(
    rocks_universal_compaction_options_t* uco, int w) {
  uco->rep.max_merge_width = w;
}

void rocks_universal_compaction_options_set_max_size_amplification_percent(
    rocks_universal_compaction_options_t* uco, int p) {
  uco->rep.max_size_amplification_percent = p;
}

void rocks_universal_compaction_options_set_compression_size_percent(
    rocks_universal_compaction_options_t* uco, int p) {
  uco->rep.compression_size_percent = p;
}

void rocks_universal_compaction_options_set_stop_style(
    rocks_universal_compaction_options_t* uco, int style) {
  uco->rep.stop_style = static_cast<rocksdb::CompactionStopStyle>(style);
}

void rocks_universal_compaction_options_destroy(
    rocks_universal_compaction_options_t* uco) {
  delete uco;
}
}
