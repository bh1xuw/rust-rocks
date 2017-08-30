#include "rocksdb/cache.h"
#include "rocksdb/persistent_cache.h"

#include "rocks/ctypes.hpp"

using namespace rocksdb;

using std::shared_ptr;

extern "C" {
rocks_cache_t* rocks_cache_create_lru(size_t capacity, int num_shard_bits, char strict_capacity_limit,
                                      double high_pri_pool_ratio) {
  rocks_cache_t* c = new rocks_cache_t;
  c->rep = NewLRUCache(capacity, num_shard_bits, strict_capacity_limit, high_pri_pool_ratio);
  return c;
}

rocks_cache_t* rocks_cache_create_clock(size_t capacity, int num_shard_bits, char strict_capacity_limit) {
  rocks_cache_t* c = new rocks_cache_t;
  c->rep = NewClockCache(capacity, num_shard_bits, strict_capacity_limit);
  if (c->rep == nullptr) {
    delete (c);
    return nullptr;
  }
  return c;
}

void rocks_cache_destroy(rocks_cache_t* cache) { delete cache; }

void rocks_cache_set_capacity(rocks_cache_t* cache, size_t capacity) { cache->rep->SetCapacity(capacity); }

size_t rocks_cache_get_capacity(rocks_cache_t* cache) { return cache->rep->GetCapacity(); }

size_t rocks_cache_get_usage(rocks_cache_t* cache) { return cache->rep->GetUsage(); }

size_t rocks_cache_get_pinned_usage(rocks_cache_t* cache) { return cache->rep->GetPinnedUsage(); }

const char* rocks_cache_name(rocks_cache_t* cache) { return cache->rep->Name(); }
}

// persistent_cache
extern "C" {
rocks_persistent_cache_t* rocks_new_persistent_cache(const rocks_env_t* env, const char* path, size_t path_len,
                                                     uint64_t size, const rocks_logger_t* log,
                                                     unsigned char optimized_for_nvm, rocks_status_t** status) {
  auto ret = new rocks_persistent_cache_t;
  auto logger = log == nullptr ? nullptr : log->rep;
  auto st = NewPersistentCache(env->rep, std::string(path, path_len), size, logger, optimized_for_nvm, &ret->rep);
  if (SaveError(status, std::move(st))) {
    delete ret;
    return nullptr;
  } else {
    return ret;
  }
}

void rocks_persistent_cache_destroy(rocks_persistent_cache_t* cache) { delete cache; }

rocks_persistent_cache_t* rocks_persistent_cache_clone(rocks_persistent_cache_t* cache) {
  return new rocks_persistent_cache_t{cache->rep};
}

cxx_string_t* rocks_persistent_cache_get_printable_options(rocks_persistent_cache_t* cache) {
  auto str = new std::string(cache->rep->GetPrintableOptions());
  return reinterpret_cast<cxx_string_t*>(str);
}
}
