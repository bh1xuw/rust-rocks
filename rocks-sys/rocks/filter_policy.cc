#include "rocksdb/filter_policy.h"

#include "rocks/ctypes.hpp"

using namespace rocksdb;

using std::unique_ptr;

extern "C" {

rocks_raw_filterpolicy_t* rocks_raw_filterpolicy_new_bloomfilter(
    int bits_per_key, unsigned char use_block_based_builder) {
  rocks_raw_filterpolicy_t* policy = new rocks_raw_filterpolicy_t;
  policy->rep.reset(
      NewBloomFilterPolicy(bits_per_key, use_block_based_builder));
  return policy;
}

void rocks_raw_filterpolicy_destroy(rocks_raw_filterpolicy_t* cache) {
  delete cache;
}
}
