#include "rocksdb/table.h"

#include "rocks/ctypes.hpp"

using namespace rocksdb;

using std::shared_ptr;

extern "C" {
rocks_plain_table_options_t* rocks_plain_table_options_create() { return new rocks_plain_table_options_t; }

void rocks_plain_table_options_destroy(rocks_plain_table_options_t* options) { delete options; }

void rocks_plain_table_options_set_user_key_len(rocks_plain_table_options_t* options, uint32_t val) {
  options->rep.user_key_len = val;
}
void rocks_plain_table_options_set_bloom_bits_per_key(rocks_plain_table_options_t* options, int val) {
  options->rep.bloom_bits_per_key = val;
}
void rocks_plain_table_options_set_hash_table_ratio(rocks_plain_table_options_t* options, double val) {
  options->rep.hash_table_ratio = val;
}
void rocks_plain_table_options_set_index_sparseness(rocks_plain_table_options_t* options, size_t val) {
  options->rep.index_sparseness = val;
}
void rocks_plain_table_options_set_huge_page_tlb_size(rocks_plain_table_options_t* options, size_t val) {
  options->rep.huge_page_tlb_size = val;
}
void rocks_plain_table_options_set_encoding_type(rocks_plain_table_options_t* options, char val) {
  options->rep.encoding_type = static_cast<EncodingType>(val);
}
void rocks_plain_table_options_set_full_scan_mode(rocks_plain_table_options_t* options, unsigned char val) {
  options->rep.full_scan_mode = val;
}
void rocks_plain_table_options_set_store_index_in_file(rocks_plain_table_options_t* options, unsigned char val) {
  options->rep.store_index_in_file = val;
}
}

extern "C" {
rocks_block_based_table_options_t* rocks_block_based_table_options_create() {
  return new rocks_block_based_table_options_t;
}

void rocks_block_based_table_options_destroy(rocks_block_based_table_options_t* options) { delete options; }

// flush_block_policy_factory

void rocks_block_based_table_options_set_cache_index_and_filter_blocks(rocks_block_based_table_options_t* options,
                                                                       unsigned char val) {
  options->rep.cache_index_and_filter_blocks = val;
}

void rocks_block_based_table_options_set_cache_index_and_filter_blocks_with_high_priority(
    rocks_block_based_table_options_t* options, unsigned char val) {
  options->rep.cache_index_and_filter_blocks_with_high_priority = val;
}

void rocks_block_based_table_options_set_pin_l0_filter_and_index_blocks_in_cache(
    rocks_block_based_table_options_t* options, unsigned char v) {
  options->rep.pin_l0_filter_and_index_blocks_in_cache = v;
}

void rocks_block_based_table_options_set_index_type(rocks_block_based_table_options_t* options, int v) {
  options->rep.index_type = static_cast<BlockBasedTableOptions::IndexType>(v);
}

void rocks_block_based_table_options_set_hash_index_allow_collision(rocks_block_based_table_options_t* options,
                                                                    unsigned char v) {
  options->rep.hash_index_allow_collision = v;
}

// checksum

void rocks_block_based_table_options_set_no_block_cache(rocks_block_based_table_options_t* options,
                                                        unsigned char no_block_cache) {
  options->rep.no_block_cache = no_block_cache;
}

void rocks_block_based_table_options_set_block_cache(rocks_block_based_table_options_t* options,
                                                     rocks_cache_t* block_cache) {
  if (block_cache) {
    options->rep.block_cache = block_cache->rep;
  }
}

// persistent_cache

void rocks_block_based_table_options_set_block_cache_compressed(rocks_block_based_table_options_t* options,
                                                                rocks_cache_t* block_cache_compressed) {
  if (block_cache_compressed) {
    options->rep.block_cache_compressed = block_cache_compressed->rep;
  }
}

void rocks_block_based_table_options_set_block_size(rocks_block_based_table_options_t* options, size_t block_size) {
  options->rep.block_size = block_size;
}

void rocks_block_based_table_options_set_block_size_deviation(rocks_block_based_table_options_t* options,
                                                              int block_size_deviation) {
  options->rep.block_size_deviation = block_size_deviation;
}

void rocks_block_based_table_options_set_block_restart_interval(rocks_block_based_table_options_t* options,
                                                                int block_restart_interval) {
  options->rep.block_restart_interval = block_restart_interval;
}

void rocks_block_based_table_options_set_index_block_restart_interval(rocks_block_based_table_options_t* options,
                                                                      int val) {
  options->rep.index_block_restart_interval = val;
}

void rocks_block_based_table_options_set_metadata_block_size(rocks_block_based_table_options_t* options, uint64_t val) {
  options->rep.metadata_block_size = val;
}

void rocks_block_based_table_options_set_partition_filters(rocks_block_based_table_options_t* options,
                                                           unsigned char val) {
  options->rep.partition_filters = val;
}

void rocks_block_based_table_options_set_use_delta_encoding(rocks_block_based_table_options_t* options,
                                                            unsigned char val) {
  options->rep.use_delta_encoding = val;
}

// TODO: customized filter policy
void rocks_block_based_table_options_set_filter_policy(rocks_block_based_table_options_t* options,
                                                       rocks_raw_filterpolicy_t* policy) {
  if (policy != nullptr) {
    options->rep.filter_policy.swap(policy->rep);
  } else {
    options->rep.filter_policy = nullptr;
  }
}

void rocks_block_based_table_options_set_whole_key_filtering(rocks_block_based_table_options_t* options,
                                                             unsigned char v) {
  options->rep.whole_key_filtering = v;
}

void rocks_block_based_table_options_set_verify_compression(rocks_block_based_table_options_t* options,
                                                            unsigned char v) {
  options->rep.verify_compression = v;
}

void rocks_block_based_table_options_set_read_amp_bytes_per_bit(rocks_block_based_table_options_t* options,
                                                                uint32_t v) {
  options->rep.read_amp_bytes_per_bit = v;
}

void rocks_block_based_table_options_set_format_version(rocks_block_based_table_options_t* options, uint32_t v) {
  options->rep.format_version = v;
}
}

extern "C" {
rocks_cuckoo_table_options_t* rocks_cuckoo_table_options_create() { return new rocks_cuckoo_table_options_t; }

void rocks_cuckoo_table_options_destroy(rocks_cuckoo_table_options_t* options) { delete options; }

void rocks_cuckoo_table_options_set_hash_table_ratio(rocks_cuckoo_table_options_t* options, double v) {
  options->rep.hash_table_ratio = v;
}

void rocks_cuckoo_table_options_set_max_search_depth(rocks_cuckoo_table_options_t* options, uint32_t v) {
  options->rep.max_search_depth = v;
}

void rocks_cuckoo_table_options_set_cuckoo_block_size(rocks_cuckoo_table_options_t* options, uint32_t v) {
  options->rep.cuckoo_block_size = v;
}

void rocks_cuckoo_table_options_set_identity_as_first_hash(rocks_cuckoo_table_options_t* options, unsigned char v) {
  options->rep.identity_as_first_hash = v;
}

void rocks_cuckoo_table_options_set_use_module_hash(rocks_cuckoo_table_options_t* options, unsigned char v) {
  options->rep.use_module_hash = v;
}
}
