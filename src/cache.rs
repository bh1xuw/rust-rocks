//! A Cache is an interface that maps keys to values.  It has internal
//! synchronization and may be safely accessed concurrently from
//! multiple threads.  It may automatically evict entries to make room
//! for new entries.  Values have a specified charge against the cache
//! capacity.  For example, a cache where the values are variable
//! length strings, may use the length of the string as the charge for
//! the string.

use rocks_sys as ll;

// #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
// pub enum Priority {
// High,
// Low,
// }
//

/// Opaque handle to an entry stored in the cache.
pub struct Handle;

/// A builtin cache implementation with a least-recently-used eviction
/// policy is provided.  Clients may use their own implementations if
/// they want something more sophisticated (like scan-resistance, a
/// custom eviction policy, variable cache sizing, etc.)
pub struct Cache {
    raw: *mut ll::rocksdb::Cache,
}

impl Cache {
    /// The type of the Cache
    pub fn name(&self) -> &'static str {
        unsafe { unimplemented!() }
    }
}

// Rust
#[derive(PartialEq, Eq)]
enum CacheType {
    LRU,
    Clock,
}

pub struct CacheBuilder {
    type_: CacheType,
    capacity: usize,
    num_shard_bits: i32,
    strict_capacity_limit: bool,
    high_pri_pool_ratio: f64,
}

impl CacheBuilder {
    /// Create a new cache with a fixed size capacity. The cache is sharded
    /// to 2^num_shard_bits shards, by hash of the key. The total capacity
    /// is divided and evenly assigned to each shard. If strict_capacity_limit
    /// is set, insert to the cache will fail when cache is full. User can also
    /// set percentage of the cache reserves for high priority entries via
    /// high_pri_pool_pct.
    /// num_shard_bits = -1 means it is automatically determined: every shard
    /// will be at least 512KB and number of shard bits will not exceed 6.
    pub fn new_lru(capacity: usize) -> CacheBuilder {
        CacheBuilder {
            type_: CacheType::LRU,
            capacity: capacity,
            num_shard_bits: -1,
            strict_capacity_limit: false,
            high_pri_pool_ratio: 0.0,
        }
    }

    /// Similar to NewLRUCache, but create a cache based on CLOCK algorithm with
    /// better concurrent performance in some cases. See util/clock_cache.cc for
    /// more detail.
    ///
    /// Return nullptr if it is not supported.
    pub fn new_clock(capacity: usize) -> CacheBuilder {
        CacheBuilder {
            type_: CacheType::Clock,
            capacity: capacity,
            num_shard_bits: -1,
            strict_capacity_limit: false,
            high_pri_pool_ratio: 0.0,
        }
    }

    pub fn build(&mut self) -> Option<Cache> {
        match self.type_ {
            CacheType::LRU => unimplemented!(),
            CacheType::Clock => unimplemented!(),
        }
    }

    pub fn num_shard_bits(&mut self, bits: i32) -> &mut Self {
        self.num_shard_bits = bits;
        self
    }

    pub fn strict_capacity_limit(&mut self, strict: bool) -> &mut Self {
        self.strict_capacity_limit = strict;
        self
    }

    pub fn high_pri_pool_ratio(&mut self, ratio: f64) -> &mut Self {
        if self.type_ == CacheType::LRU {
            self.high_pri_pool_ratio = ratio
        } else {
            panic!("ClockCache doesn't support high_pri_pool_ratio")
        }
        self
    }
}
