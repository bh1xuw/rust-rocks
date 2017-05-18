#include "rocksdb/comparator.h"

#include "rocks/ctypes.hpp"


using namespace rocksdb;

extern "C" {
  const Comparator* rocks_comparater_bytewise() {
    return BytewiseComparator();
  }

  const Comparator* rocks_comparater_bytewise_reversed() {
    return ReverseBytewiseComparator();
  }
}
