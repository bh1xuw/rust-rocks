//! Table options.
//!
//! Currently we support two types of tables: plain table and block-based table.
//! 
//! 1. Block-based table: this is the default table type that we inherited from
//!    LevelDB, which was designed for storing data in hard disk or flash
//!    device.
//! 2. Plain table: it is one of RocksDB's SST file format optimized
//!    for low query latency on pure-memory or really low-latency media.
//!
//! A tutorial of rocksdb table formats is available here:
//! >  https://github.com/facebook/rocksdb/wiki/A-Tutorial-of-RocksDB-SST-formats
//!
//! Example code is also available
//! > https://github.com/facebook/rocksdb/wiki/A-Tutorial-of-RocksDB-SST-formats#wiki-examples

use std::mem;
use std::ptr;
use std::os::raw::c_int;

use rocks_sys as ll;

use cache::Cache;
use to_raw::ToRaw;
use filter_policy::FilterPolicy;

#[repr(C)]
pub enum IndexType {
    /// A space efficient index block that is optimized for
    /// binary-search-based index.
    BinarySearch,

    /// The hash index, if enabled, will do the hash lookup when
    /// `Options.prefix_extractor` is provided.
    HashSearch,

    /// TODO(myabandeh): this feature is in experimental phase and shall not be
    /// used in production; either remove the feature or remove this comment if
    /// it is ready to be used in production.
    /// 
    /// A two-level index implementation. Both levels are binary search indexes.
    TwoLevelIndexSearch,
}

/// For advanced user only
pub struct BlockBasedTableOptions {
    raw: *mut ll::rocks_block_based_table_options_t,
}

impl Default for BlockBasedTableOptions {
    fn default() -> Self {
        BlockBasedTableOptions {
            raw: unsafe { ll::rocks_block_based_table_options_create() },
        }
    }
}

impl ToRaw<ll::rocks_block_based_table_options_t> for BlockBasedTableOptions {
    fn raw(&self) -> *mut ll::rocks_block_based_table_options_t {
        self.raw
    }
}

impl BlockBasedTableOptions {
    // `flush_block_policy_factory` creates the instances of flush block policy.
    // which provides a configurable way to determine when to flush a block in
    // the block based tables.  If not set, table builder will use the default
    // block flush policy, which cut blocks by block size (please refer to
    // `FlushBlockBySizePolicy`).
    //
    // std::shared_ptr<FlushBlockPolicyFactory> ;
    // pub fn flush_block_policy_factory(self, v: ()) -> Self {
    //     unimplemented!()
    // }

    /// TODO(kailiu) Temporarily disable this feature by making the default value
    /// to be false.
    ///
    /// Indicating if we'd put index/filter blocks to the block cache.
    /// 
    /// If not specified, each "table reader" object will pre-load index/filter
    /// block during table initialization.
    pub fn cache_index_and_filter_blocks(self, val: bool) -> Self {
        unsafe {
            ll::rocks_block_based_table_options_set_cache_index_and_filter_blocks(self.raw, val as u8);
        }
        self
    }

    /// If `cache_index_and_filter_blocks` is enabled, cache index and filter
    /// blocks with high priority. If set to true, depending on implementation of
    /// block cache, index and filter blocks may be less likely to be evicted
    /// than data blocks.
    pub fn cache_index_and_filter_blocks_with_high_priority(self, val: bool) -> Self {
        unsafe {
            ll::rocks_block_based_table_options_set_cache_index_and_filter_blocks_with_high_priority(self.raw, val as u8);
        }
        self
    }

    /// if `cache_index_and_filter_blocks` is true and the below is true, then
    /// filter and index blocks are stored in the cache, but a reference is
    /// held in the "table reader" object so the blocks are pinned and only
    /// evicted from cache when the table reader is freed.
    pub fn pin_l0_filter_and_index_blocks_in_cache(self, val: bool) -> Self {
        unsafe {
            ll::rocks_block_based_table_options_set_pin_l0_filter_and_index_blocks_in_cache(self.raw, val as u8);
        }
        self
    }

    pub fn index_type(self, val: IndexType) -> Self {
        unsafe {
            ll::rocks_block_based_table_options_set_index_type(self.raw, mem::transmute(val))
        }
        self
    }

    /// This option is now deprecated. No matter what value it is set to,
    /// it will behave as if `hash_index_allow_collision=true`.
    pub fn hash_index_allow_collision(self, val: bool) -> Self {
        unsafe {
            ll::rocks_block_based_table_options_set_hash_index_allow_collision(self.raw, val as u8);
        }
        self
    }

    // Use the specified checksum type. Newly created table files will be
    // protected with this checksum type. Old table files will still be readable,
    // even though they have different checksum type.
    //
    // ChecksumType checksum = kCRC32c;

    /// Disable block cache. If this is set to true,
    /// then no block cache should be used, and the block_cache should
    /// point to a nullptr object.
    pub fn no_block_cache(self, val: bool) -> Self {
        unsafe {
            ll::rocks_block_based_table_options_set_no_block_cache(self.raw, val as u8);
        }
        self
    }

    /// If non-NULL use the specified cache for blocks.
    ///
    /// If NULL, rocksdb will automatically create and use an 8MB internal cache.
    pub fn block_cache(self, val: Option<Cache>) -> Self {
        unsafe {
            let ptr = val.map(|c| c.raw()).unwrap_or_else(ptr::null_mut);
            ll::rocks_block_based_table_options_set_block_cache(self.raw, ptr);
        }
        self
    }

    // If non-NULL use the specified cache for pages read from device
    // IF NULL, no page cache is used
    //
    // std::shared_ptr<PersistentCache> persistent_cache = nullptr;

    /// If non-NULL use the specified cache for compressed blocks.
    /// 
    /// If NULL, rocksdb will not use a compressed block cache.
    pub fn block_cache_compressed(self, val: Option<Cache>) -> Self {
        unsafe {
            let ptr = val.map(|c| c.raw()).unwrap_or_else(ptr::null_mut);
            ll::rocks_block_based_table_options_set_block_cache_compressed(self.raw, ptr);
        }
        self
    }

    /// Approximate size of user data packed per block.  Note that the
    /// block size specified here corresponds to uncompressed data.  The
    /// actual size of the unit read from disk may be smaller if
    /// compression is enabled.  This parameter can be changed dynamically.
    pub fn block_size(self, val: usize) -> Self {
        unsafe {
            ll::rocks_block_based_table_options_set_block_size(self.raw, val);
        }
        self
    }

    /// This is used to close a block before it reaches the configured
    /// 'block_size'. If the percentage of free space in the current block is less
    /// than this specified number and adding a new record to the block will
    /// exceed the configured block size, then this block will be closed and the
    /// new record will be written to the next block.
    pub fn block_size_deviation(self, val: i32) -> Self {
        unsafe {
            ll::rocks_block_based_table_options_set_block_size_deviation(self.raw, val as c_int);
        }
        self
    }

    /// Number of keys between restart points for delta encoding of keys.
    /// This parameter can be changed dynamically.  Most clients should
    /// leave this parameter alone.  The minimum value allowed is 1.  Any smaller
    /// value will be silently overwritten with 1.
    pub fn block_restart_interval(self, val: i32) -> Self {
        unsafe {
            ll::rocks_block_based_table_options_set_block_restart_interval(self.raw, val);
        }
        self
    }

    /// Same as `block_restart_interval` but used for the index block.
    pub fn index_block_restart_interval(self, val: i32) -> Self {
        unsafe {
            ll::rocks_block_based_table_options_set_index_block_restart_interval(self.raw, val);
        }
        self
    }

    /// Block size for partitioned metadata. Currently applied to indexes when
    /// `kTwoLevelIndexSearch` is used and to filters when partition_filters is used.
    /// 
    /// Note: Since in the current implementation the filters and index partitions
    /// are aligned, an index/filter block is created when either index or filter
    /// block size reaches the specified limit.
    ///
    /// Note: this limit is currently applied to only index blocks; a filter
    /// partition is cut right after an index block is cut
    ///
    /// TODO(myabandeh): remove the note above when filter partitions are cut
    /// separately
    pub fn metadata_block_size(self, val: u64) -> Self {
        // unsafe {
        //     ll::rocks_block_based_table_options_set_metadata_block_size(self.raw, val);
        // }
        // self
        unimplemented!()        // FIXME: in 5.4
    }

    /// Note: currently this option requires kTwoLevelIndexSearch to be set as
    /// well.
    ///
    /// TODO(myabandeh): remove the note above once the limitation is lifted
    ///
    /// TODO(myabandeh): this feature is in experimental phase and shall not be
    /// used in production; either remove the feature or remove this comment if
    /// it is ready to be used in production.
    ///
    /// Use partitioned full filters for each SST file
    pub fn partition_filters(self, val: bool) -> Self {
        unsafe {
            ll::rocks_block_based_table_options_set_partition_filters(self.raw, val as u8);
        }
        self
    }

    /// Use delta encoding to compress keys in blocks.
    /// `ReadOptions::pin_data` requires this option to be disabled.
    ///
    /// Default: true
    pub fn use_delta_encoding(self, val: bool) -> Self {
        unsafe {
            ll::rocks_block_based_table_options_set_use_delta_encoding(self.raw, val as u8);
        }
        self
    }

    /// If non-nullptr, use the specified filter policy to reduce disk reads.
    ///
    /// Many applications will benefit from passing the result of
    /// `NewBloomFilterPolicy()` here.
    pub fn filter_policy(self, val: Option<FilterPolicy>) -> Self {
        if let Some(policy) = val {
            unsafe {
                ll::rocks_block_based_table_options_set_filter_policy(self.raw, policy.raw());
            }
        } else {
            unsafe {
                ll::rocks_block_based_table_options_set_filter_policy(self.raw, ptr::null_mut());
            }
        }
        self
    }

    /// If true, place whole keys in the filter (not just prefixes).
    /// This must generally be true for gets to be efficient.
    pub fn whole_key_filtering(self, val: bool) -> Self {
        unsafe {
            ll::rocks_block_based_table_options_set_whole_key_filtering(self.raw, val as u8);
        }
        self
    }

    /// Verify that decompressing the compressed block gives back the input. This
    /// is a verification mode that we use to detect bugs in compression
    /// algorithms.
    pub fn verify_compression(self, val: bool) -> Self {
        unsafe {
            ll::rocks_block_based_table_options_set_verify_compression(self.raw, val as u8);
        }
        self
    }

    /// If used, For every data block we load into memory, we will create a bitmap
    /// of size ((block_size / `read_amp_bytes_per_bit`) / 8) bytes. This bitmap
    /// will be used to figure out the percentage we actually read of the blocks.
    ///
    /// When this feature is used Tickers::READ_AMP_ESTIMATE_USEFUL_BYTES and
    /// Tickers::READ_AMP_TOTAL_READ_BYTES can be used to calculate the
    /// read amplification using this formula
    /// (READ_AMP_TOTAL_READ_BYTES / READ_AMP_ESTIMATE_USEFUL_BYTES)
    ///
    /// value  =>  memory usage (percentage of loaded blocks memory)
    /// 1      =>  12.50 %
    /// 2      =>  06.25 %
    /// 4      =>  03.12 %
    /// 8      =>  01.56 %
    /// 16     =>  00.78 %
    ///
    /// Note: This number must be a power of 2, if not it will be sanitized
    /// to be the next lowest power of 2, for example a value of 7 will be
    /// treated as 4, a value of 19 will be treated as 16.
    ///
    /// Default: 0 (disabled)
    pub fn read_amp_bytes_per_bit(self, val: u32) -> Self {
        unsafe {
            ll::rocks_block_based_table_options_set_read_amp_bytes_per_bit(self.raw, val);
        }
        self
    }

    /// We currently have three versions:
    ///
    /// 0 -- This version is currently written out by all RocksDB's versions by
    /// default.  Can be read by really old RocksDB's. Doesn't support changing
    /// checksum (default is CRC32).
    ///
    /// 1 -- Can be read by RocksDB's versions since 3.0. Supports non-default
    /// checksum, like xxHash. It is written by RocksDB when
    /// BlockBasedTableOptions::checksum is something other than kCRC32c. (version
    /// 0 is silently upconverted)
    ///
    /// 2 -- Can be read by RocksDB's versions since 3.10. Changes the way we
    /// encode compressed blocks with LZ4, BZip2 and Zlib compression. If you
    /// don't plan to run RocksDB before version 3.10, you should probably use
    /// this.
    ///
    /// This option only affects newly written tables. When reading exising tables,
    /// the information about version is read from the footer.
    pub fn format_version(self, val: u32) -> Self {
        unsafe {
            ll::rocks_block_based_table_options_set_format_version(self.raw, val);
        }
        self
    }
}


#[repr(u8)]
pub enum EncodingType {
    /// Always write full keys without any special encoding.
    Plain,
    /// Find opportunity to write the same prefix once for multiple rows.
    /// In some cases, when a key follows a previous key with the same prefix,
    /// instead of writing out the full key, it just writes out the size of the
    /// shared prefix, as well as other bytes, to save some bytes.
    ///
    /// When using this option, the user is required to use the same prefix
    /// extractor to make sure the same prefix will be extracted from the same key.
    /// The Name() value of the prefix extractor will be stored in the file. When
    /// reopening the file, the name of the options.prefix_extractor given will be
    /// bitwise compared to the prefix extractors stored in the file. An error
    /// will be returned if the two don't match.
    Prefix,
}

pub struct PlainTableOptions {
    raw: *mut ll::rocks_plain_table_options_t,
}

impl Default for PlainTableOptions {
    fn default() -> Self {
        PlainTableOptions {
            raw: unsafe { ll::rocks_plain_table_options_create() },
        }
    }
}

impl ToRaw<ll::rocks_plain_table_options_t> for PlainTableOptions {
    fn raw(&self) -> *mut ll::rocks_plain_table_options_t {
        self.raw
    }
}

impl PlainTableOptions {
    /// @user_key_len: plain table has optimization for fix-sized keys, which can
    ///                be specified via user_key_len.  Alternatively, you can pass
    ///                `kPlainTableVariableLength` if your keys have variable
    ///                lengths.
    pub fn user_key_len(self, val: u32) -> Self {
        unsafe {
            ll::rocks_plain_table_options_set_user_key_len(self.raw, val);
        }
        self
    }

    /// @bloom_bits_per_key: the number of bits used for bloom filer per prefix.
    ///                      You may disable it by passing a zero.
    pub fn bloom_bits_per_key(self, val: i32) -> Self {
        unsafe {
            ll::rocks_plain_table_options_set_bloom_bits_per_key(self.raw, val);
        }
        self
    }

    /// @hash_table_ratio: the desired utilization of the hash table used for
    ///                    prefix hashing.
    ///                    hash_table_ratio = number of prefixes / #buckets in the
    ///                    hash table
    pub fn hash_table_ratio(self, val: f64) -> Self {
        unsafe {
            ll::rocks_plain_table_options_set_hash_table_ratio(self.raw, val);
        }
        self
    }

    /// @index_sparseness: inside each prefix, need to build one index record for
    ///                    how many keys for binary search inside each hash bucket.
    ///                    For encoding type kPrefix, the value will be used when
    ///                    writing to determine an interval to rewrite the full
    ///                    key. It will also be used as a suggestion and satisfied
    ///                    when possible.
    pub fn index_sparseness(self, val: usize) -> Self {
        unsafe {
            ll::rocks_plain_table_options_set_index_sparseness(self.raw, val);
        }
        self
    }

    /// @huge_page_tlb_size: if <=0, allocate hash indexes and blooms from malloc.
    ///                      Otherwise from huge page TLB. The user needs to
    ///                      reserve huge pages for it to be allocated, like:
    ///                          sysctl -w vm.nr_hugepages=20
    ///                      See linux doc Documentation/vm/hugetlbpage.txt
    pub fn huge_page_tlb_size(self, val: usize) -> Self {
        unsafe {
            ll::rocks_plain_table_options_set_huge_page_tlb_size(self.raw, val);
        }
        self
    }

    /// @encoding_type: how to encode the keys. See enum EncodingType above for
    ///                 the choices. The value will determine how to encode keys
    ///                 when writing to a new SST file. This value will be stored
    ///                 inside the SST file which will be used when reading from
    ///                 the file, which makes it possible for users to choose
    ///                 different encoding type when reopening a DB. Files with
    ///                 different encoding types can co-exist in the same DB and
    ///                 can be read.
    pub fn encoding_type(self, val: EncodingType) -> Self {
        unsafe {
            ll::rocks_plain_table_options_set_encoding_type(self.raw, mem::transmute(val));
        }
        self
    }

    /// @full_scan_mode: mode for reading the whole file one record by one without
    ///                  using the index.
    pub fn full_scan_mode(self, val: bool) -> Self {
        unsafe {
            ll::rocks_plain_table_options_set_full_scan_mode(self.raw, val as u8);
        }
        self
    }

    /// @store_index_in_file: compute plain table index and bloom filter during
    ///                       file building and store it in file. When reading
    ///                       file, index will be mmaped instead of recomputation.
    pub fn store_index_in_file(self, val: bool) -> Self {
        unsafe {
            ll::rocks_plain_table_options_set_store_index_in_file(self.raw, val as u8);
        }
        self
    }
}


pub struct CuckooTableOptions {
    raw: *mut ll::rocks_cuckoo_table_options_t,
}

impl Default for CuckooTableOptions {
    fn default() -> Self {
        CuckooTableOptions {
            raw: unsafe { ll::rocks_cuckoo_table_options_create() },
        }
    }
}

impl ToRaw<ll::rocks_cuckoo_table_options_t> for CuckooTableOptions {
    fn raw(&self) -> *mut ll::rocks_cuckoo_table_options_t {
        self.raw
    }
}

impl CuckooTableOptions {
    /// Determines the utilization of hash tables. Smaller values
    /// result in larger hash tables with fewer collisions.
    pub fn hash_table_ratio(self, val: f64) -> Self {
        unsafe {
            ll::rocks_cuckoo_table_options_set_hash_table_ratio(self.raw, val);
        }
        self
    }
    /// A property used by builder to determine the depth to go to
    /// to search for a path to displace elements in case of
    /// collision. See Builder.MakeSpaceForKey method. Higher
    /// values result in more efficient hash tables with fewer
    /// lookups but take more time to build.
    pub fn max_search_depth(self, val: u32) -> Self {
        unsafe {
            ll::rocks_cuckoo_table_options_set_max_search_depth(self.raw, val);
        }
        self
    }
    /// In case of collision while inserting, the builder
    /// attempts to insert in the next cuckoo_block_size
    /// locations before skipping over to the next Cuckoo hash
    /// function. This makes lookups more cache friendly in case
    /// of collisions.
    pub fn cuckoo_block_size(self, val: u32) -> Self {
        unsafe {
            ll::rocks_cuckoo_table_options_set_cuckoo_block_size(self.raw, val);
        }
        self
    }
    /// If this option is enabled, user key is treated as uint64_t and its value
    /// is used as hash value directly. This option changes builder's behavior.
    /// Reader ignore this option and behave according to what specified in table
    /// property.
    pub fn identity_as_first_hash(self, val: bool) -> Self {
        unsafe {
            ll::rocks_cuckoo_table_options_set_identity_as_first_hash(self.raw, val as u8);
        }
        self
    }
    /// If this option is set to true, module is used during hash calculation.
    /// This often yields better space efficiency at the cost of performance.
    /// If this optino is set to false, # of entries in table is constrained to be
    /// power of two, and bit and is used to calculate hash, which is faster in
    /// general.
    pub fn use_module_hash(self, val: bool) -> Self {
        unsafe {
            ll::rocks_cuckoo_table_options_set_use_module_hash(self.raw, val as u8);
        }
        self
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        
    }
}
