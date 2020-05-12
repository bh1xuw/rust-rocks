#include "rocksdb/statistics.h"

#include <algorithm>

#include "rocks/ctypes.hpp"
#include "rust_export.h"

using namespace rocksdb;

extern "C" {
rocks_statistics_t* rocks_statistics_create() { return new rocks_statistics_t{CreateDBStatistics()}; }

// FIXME: is this naming right?
rocks_statistics_t* rocks_statistics_copy(rocks_statistics_t* stat) {
  auto new_rep = stat->rep;
  return new rocks_statistics_t{new_rep};
}

void rocks_statistics_destroy(rocks_statistics_t* stat) { delete stat; }

uint64_t rocks_statistics_get_ticker_count(rocks_statistics_t* stat, const char* key, size_t key_len) {
  auto ticker_name = std::string(key, key_len);
  auto it = std::find_if(TickersNameMap.begin(), TickersNameMap.end(),
                         [&](std::pair<Tickers, std::string> pair) { return pair.second == ticker_name; });
  if (it != TickersNameMap.end()) {
    auto ticker_type = it->first;
    return stat->rep->getTickerCount(ticker_type);
  } else {
    return 0;
  }
}

void rocks_statistics_histogram_data(rocks_statistics_t* stat, const char* key, size_t key_len,
                                     rocks_histogram_data_t* const data) {
  auto histo_name = std::string(key, key_len);
  auto it = std::find_if(HistogramsNameMap.begin(), HistogramsNameMap.end(),
                         [&](std::pair<Histograms, std::string> pair) { return pair.second == histo_name; });
  if (it != HistogramsNameMap.end()) {
    auto histo_type = it->first;
    stat->rep->histogramData(histo_type, reinterpret_cast<HistogramData* const>(data));
  }
}

void rocks_statistics_get_histogram_string(rocks_statistics_t* stat, const char* key, size_t key_len, void* str) {
  auto histo_name = std::string(key, key_len);
  auto it = std::find_if(HistogramsNameMap.begin(), HistogramsNameMap.end(),
                         [&](std::pair<Histograms, std::string> pair) { return pair.second == histo_name; });
  if (it != HistogramsNameMap.end()) {
    auto s = stat->rep->getHistogramString(it->first);
    rust_string_assign(str, s.data(), s.size());
  }
}

void rocks_statistics_record_tick(rocks_statistics_t* stat, uint32_t tickerType, uint64_t count) {
  stat->rep->recordTick(tickerType, count);
}

void rocks_statistics_set_ticker_count(rocks_statistics_t* stat, uint32_t tickerType, uint64_t count) {
  stat->rep->setTickerCount(tickerType, count);
}

uint64_t rocks_statistics_get_and_reset_ticker_count(rocks_statistics_t* stat, const char* key, size_t key_len) {
    auto ticker_name = std::string(key, key_len);
  auto it = std::find_if(TickersNameMap.begin(), TickersNameMap.end(),
                         [&](std::pair<Tickers, std::string> pair) { return pair.second == ticker_name; });
  if (it != TickersNameMap.end()) {
    auto ticker_type = it->first;
    return stat->rep->getAndResetTickerCount(ticker_type);
  } else {
    return 0;
  }
}

void rocks_statistics_measure_time(rocks_statistics_t* stat, uint32_t histogramType, uint64_t time) {
  stat->rep->measureTime(histogramType, time);
}

// *mut String
void rocks_statistics_to_string(rocks_statistics_t* stat, void* str) {
  auto s = stat->rep->ToString();
  rust_string_assign(str, s.data(), s.size());
}

unsigned char rocks_statistics_hist_enabled_for_type(rocks_statistics_t* stat, uint32_t type) {
  return stat->rep->HistEnabledForType(type);
}

void rocks_statistics_reset(rocks_statistics_t* stat, rocks_status_t** status) {
  SaveError(status, stat->rep->Reset());
}
}
