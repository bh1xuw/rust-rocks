#include "rocksdb/comparator.h"

#include "rocks/ctypes.hpp"

using namespace rocksdb;

extern "C" {
const Comparator* rocks_comparator_bytewise() { return BytewiseComparator(); }

const Comparator* rocks_comparator_bytewise_reversed() { return ReverseBytewiseComparator(); }
}
