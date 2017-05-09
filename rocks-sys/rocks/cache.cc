#include "rocksdb/cache.h"

#include "rocks/ctypes.hpp"


using namespace rocksdb;

using std::shared_ptr;

extern "C" {
  rocks_cache_t* rocks_cache_create_lru(size_t capacity, int num_shard_bits, char strict_capacity_limit, double high_pri_pool_ratio) {
    rocks_cache_t* c = new rocks_cache_t;
    c->rep = NewLRUCache(capacity, num_shard_bits, strict_capacity_limit, high_pri_pool_ratio);
    return c;
  }

  rocks_cache_t* rocks_cache_create_clock(size_t capacity, int num_shard_bits, char strict_capacity_limit) {
    rocks_cache_t* c = new rocks_cache_t;
    c->rep = NewClockCache(capacity, num_shard_bits, strict_capacity_limit);
    if (c->rep == nullptr) {
      delete(c);
      return nullptr;
    }
    return c;
  }

  void rocks_cache_destroy(rocks_cache_t* cache) {
    delete cache;
  }

  void rocks_cache_set_capacity(rocks_cache_t* cache, size_t capacity) {
    cache->rep->SetCapacity(capacity);
  }

  size_t rocks_cache_get_capacity(rocks_cache_t* cache) {
    return cache->rep->GetCapacity();
  }

  size_t rocks_cache_get_usage(rocks_cache_t* cache) {
    return cache->rep->GetUsage();
  }

  size_t rocks_cache_get_pinned_usage(rocks_cache_t* cache) {
    return cache->rep->GetPinnedUsage();
  }

  const char* rocks_cache_name(rocks_cache_t* cache) {
    return cache->rep->Name();
  }
}
