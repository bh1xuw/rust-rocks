#include "rocksdb/statistics.h"

#include "rocks/ctypes.hpp"

#include "rust_export.h"

using namespace rocksdb;

extern "C" {
  rocks_statistics_t* rocks_statistics_create() {
    return new rocks_statistics_t { CreateDBStatistics() };
  }

  // FIXME: is this naming right?
  rocks_statistics_t* rocks_statistics_copy(rocks_statistics_t* stat) {
    auto new_rep = stat->rep;
    return new rocks_statistics_t { new_rep };
  }

  void rocks_statistics_destroy(rocks_statistics_t* stat) {
    delete stat;
  }

  uint64_t rocks_statistics_get_ticker_count(rocks_statistics_t* stat, uint32_t tickerType) {
    return stat->rep->getTickerCount(tickerType);
  }

  void rocks_statistics_histogram_data(rocks_statistics_t* stat,
                                       uint32_t type,
                                       rocks_histogram_data_t* const data) {
    stat->rep->histogramData(type, reinterpret_cast<HistogramData* const>(data));
  }

  void rocks_statistics_get_histogram_string(rocks_statistics_t* stat,
                                             uint32_t type,
                                             void* str) { // *mut String
    auto s = stat->rep->getHistogramString(type);
    rust_string_assign(str, s.data(), s.size());
  }

  void rocks_statistics_record_tick(rocks_statistics_t* stat,
                                    uint32_t tickerType,
                                    uint64_t count) {
    stat->rep->recordTick(tickerType, count);
  }

  void rocks_statistics_set_ticker_count(rocks_statistics_t* stat,
                                         uint32_t tickerType,
                                         uint64_t count) {
    stat->rep->setTickerCount(tickerType, count);
  }

  uint64_t rocks_statistics_get_and_reset_ticker_count(rocks_statistics_t* stat,
                                                       uint32_t tickerType) {
    return stat->rep->getAndResetTickerCount(tickerType);
  }

  void rocks_statistics_measure_time(rocks_statistics_t* stat,
                                     uint32_t histogramType,
                                     uint64_t time) {
    stat->rep->measureTime(histogramType, time);
  }

  void rocks_statistics_to_string(rocks_statistics_t* stat,
                                  void* str) {// *mut String
    auto s = stat->rep->ToString();
    rust_string_assign(str, s.data(), s.size());
  }

  unsigned char rocks_statistics_hist_enabled_for_type(rocks_statistics_t* stat, uint32_t type) {
    return stat->rep->HistEnabledForType(type);
  }
}



