//! Common options for DB, CF, read/write/flush/compact...

use std::u64;
use std::path::{Path, PathBuf};
use std::mem;
use std::ptr;
use std::fmt;
use std::slice;
use std::str;
use std::os::raw::c_int;
use std::marker::PhantomData;

use rocks_sys as ll;

use env::{Env, InfoLogLevel, Logger};
use listener::EventListener;
use write_buffer_manager::WriteBufferManager;
use rate_limiter::RateLimiter;
use sst_file_manager::SstFileManager;
use statistics::Statistics;
use cache::Cache;
use advanced_options::{CompactionOptionsFIFO, CompactionPri, CompactionStyle, CompressionOptions};
use universal_compaction::CompactionOptionsUniversal;
use compaction_filter::{CompactionFilter, CompactionFilterFactory};
use merge_operator::{AssociativeMergeOperator, MergeOperator};
use table::{BlockBasedTableOptions, CuckooTableOptions, PlainTableOptions};
use comparator::Comparator;
use slice_transform::SliceTransform;
use snapshot::Snapshot;
use table_properties::TablePropertiesCollectorFactory;

use to_raw::{FromRaw, ToRaw};

lazy_static! {
    // since all Options field are guaranteed to be thread safe
    static ref DEFAULT_OPTIONS: Options = {
        Options::default().map_db_options(|db| db.create_if_missing(true))
    };
    static ref DEFAULT_READ_OPTIONS: ReadOptions<'static> = {
        ReadOptions::default()
    };
    static ref DEFAULT_WRITE_OPTIONS: WriteOptions = {
        WriteOptions::default()
    };
}


/// DB contents are stored in a set of blocks, each of which holds a
/// sequence of key,value pairs.  Each block may be compressed before
/// being stored in a file.  The following enum describes which
/// compression method (if any) is used to compress a block.
#[repr(C)]
// FIXME: u8 in rocksdb
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CompressionType {
    /// NOTE: do not change the values of existing entries, as these are
    /// part of the persistent format on disk.
    NoCompression = 0x0,
    SnappyCompression = 0x1,
    ZlibCompression = 0x2,
    BZip2Compression = 0x3,
    LZ4Compression = 0x4,
    LZ4HCCompression = 0x5,
    XpressCompression = 0x6,
    ZSTD = 0x7,

    /// Only use kZSTDNotFinalCompression if you have to use ZSTD lib older than
    /// 0.8.0 or consider a possibility of downgrading the service or copying
    /// the database files to another service running with an older version of
    /// RocksDB that doesn't have kZSTD. Otherwise, you should use kZSTD. We will
    /// eventually remove the option from the public API.
    ZSTDNotFinalCompression = 0x40,

    /// kDisableCompressionOption is used to disable some compression options.
    DisableCompressionOption = 0xff,
}

/// Recovery mode to control the consistency while replaying WAL
#[repr(C)]
pub enum WALRecoveryMode {
    /// Original levelDB recovery
    /// We tolerate incomplete record in trailing data on all logs
    /// Use case : This is legacy behavior (default)
    TolerateCorruptedTailRecords = 0x00,
    /// Recover from clean shutdown
    /// We don't expect to find any corruption in the WAL
    /// Use case : This is ideal for unit tests and rare applications that
    /// can require high consistency guarantee
    AbsoluteConsistency = 0x01,
    /// Recover to point-in-time consistency
    /// We stop the WAL playback on discovering WAL inconsistency
    /// Use case : Ideal for systems that have disk controller cache like
    /// hard disk, SSD without super capacitor that store related data
    PointInTimeRecovery = 0x02,
    /// Recovery after a disaster
    /// We ignore any corruption in the WAL and try to salvage as much data as
    /// possible
    /// Use case : Ideal for last ditch effort to recover data or systems that
    /// operate with low grade unrelated data
    SkipAnyCorruptedRecords = 0x03,
}


pub struct DbPath {
    pub path: PathBuf,
    /// Target size of total files under the path, in byte.
    pub target_size: u64,
}

impl DbPath {
    pub fn new<P: AsRef<Path>>(p: P, t: u64) -> DbPath {
        DbPath {
            path: p.as_ref().to_path_buf(),
            target_size: t,
        }
    }
}

impl Default for DbPath {
    fn default() -> Self {
        DbPath::new("", 0)
    }
}

impl<T: Into<PathBuf>> From<T> for DbPath {
    fn from(path: T) -> DbPath {
        DbPath {
            path: path.into(),
            target_size: 0,
        }
    }
}

/* impl<P: Into<PathBuf>, S: Into<u64>> From<(P, S)> for DbPath {
    fn from((path, size): (P, S)) -> DbPath {
        DbPath {
            path: path.into(),
            target_size: size.into(),
        }
    }
} */

/// Options for a column family
pub struct ColumnFamilyOptions {
    raw: *mut ll::rocks_cfoptions_t,
}

impl ToRaw<ll::rocks_cfoptions_t> for ColumnFamilyOptions {
    fn raw(&self) -> *mut ll::rocks_cfoptions_t {
        self.raw
    }
}

impl Default for ColumnFamilyOptions {
    fn default() -> Self {
        ColumnFamilyOptions { raw: unsafe { ll::rocks_cfoptions_create() } }
    }
}

impl Drop for ColumnFamilyOptions {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_cfoptions_destroy(self.raw);
        }
    }
}

impl fmt::Display for ColumnFamilyOptions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            let cxx_string = ll::rocks_get_string_from_cfoptions(self.raw);
            let len = ll::cxx_string_size(cxx_string);
            let base = ll::cxx_string_data(cxx_string);
            if !cxx_string.is_null() {
                let str_rep = str::from_utf8_unchecked(slice::from_raw_parts(base as *const u8, len));
                try!(f.write_str(str_rep));
                ll::cxx_string_destroy(cxx_string);
                Ok(())
            } else {
                f.write_str("ColumnFamilyOptions { error while converting to String }")
            }
        }
    }
}

impl ColumnFamilyOptions {
    /// Create ColumnFamilyOptions with default values for all fields
    pub fn new() -> ColumnFamilyOptions {
        ColumnFamilyOptions { raw: unsafe { ll::rocks_cfoptions_create() } }
    }

    unsafe fn from_ll(raw: *mut ll::rocks_cfoptions_t) -> ColumnFamilyOptions {
        ColumnFamilyOptions { raw: raw }
    }

    pub fn from_options(opt: &Options) -> ColumnFamilyOptions {
        ColumnFamilyOptions { raw: unsafe { ll::rocks_cfoptions_create_from_options(opt.raw()) } }
    }

    /// Some functions that make it easier to optimize RocksDB

    /// Use this if your DB is very small (like under 1GB) and you don't want to
    /// spend lots of memory for memtables.
    pub fn optimize_for_small_db(self) -> Self {
        unsafe {
            ll::rocks_cfoptions_optimize_for_small_db(self.raw);
        }
        self
    }

    /// Use this if you don't need to keep the data sorted, i.e. you'll never use
    /// an iterator, only Put() and Get() API calls
    ///
    /// Not supported in ROCKSDB_LITE
    pub fn optimize_for_point_lookup(self, block_cache_size_mb: u64) -> Self {
        unsafe { ll::rocks_cfoptions_optimize_for_point_lookup(self.raw, block_cache_size_mb) }
        self
    }

    /// Default values for some parameters in ColumnFamilyOptions are not
    /// optimized for heavy workloads and big datasets, which means you might
    /// observe write stalls under some conditions. As a starting point for tuning
    /// RocksDB options, use the following two functions:
    ///
    /// * OptimizeLevelStyleCompaction -- optimizes level style compaction
    /// * OptimizeUniversalStyleCompaction -- optimizes universal style compaction
    ///
    /// Universal style compaction is focused on reducing Write Amplification
    /// Factor for big data sets, but increases Space Amplification. You can learn
    /// more about the different styles here:
    /// https://github.com/facebook/rocksdb/wiki/Rocksdb-Architecture-Guide
    /// Make sure to also call IncreaseParallelism(), which will provide the
    /// biggest performance gains.
    ///
    /// Note: we might use more memory than memtable_memory_budget during high
    /// write rate period
    ///
    /// OptimizeUniversalStyleCompaction is not supported in ROCKSDB_LITE
    pub fn optimize_level_style_compaction(self, memtable_memory_budget: u64) -> Self {
        // 512 * 1024 * 1024
        unsafe {
            ll::rocks_cfoptions_optimize_level_style_compaction(self.raw, memtable_memory_budget);
        }
        self
    }

    pub fn optimize_universal_style_compaction(self, memtable_memory_budget: u64) -> Self {
        // 512 * 1024 * 1024
        unsafe {
            ll::rocks_cfoptions_optimize_universal_style_compaction(self.raw, memtable_memory_budget);
        }
        self
    }

    // Parameters that affect behavior

    /// Comparator used to define the order of keys in the table.
    /// Default: a comparator that uses lexicographic byte-wise ordering
    ///
    /// REQUIRES: The client must ensure that the comparator supplied
    /// here has the same name and orders keys *exactly* the same as the
    /// comparator provided to previous open calls on the same DB.
    pub fn comparator<T: Comparator>(self, val: &'static T) -> Self {
        unsafe {
            // FIXME: mem leaks, CFOptions.comparator is a raw pointer,
            // not a shared_ptr
            let raw_ptr = Box::into_raw(Box::new(val as &Comparator));
            ll::rocks_cfoptions_set_comparator_by_trait(self.raw, raw_ptr as *mut _);
        }
        self
    }

    /// rust-rocks extension.
    ///
    /// use bitwise comparator and set if reversed.
    pub fn bitwise_comparator_reversed(self, val: bool) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_bitwise_comparator(self.raw, val as u8);
        }
        self
    }

    /// REQUIRES: The client must provide a merge operator if Merge operation
    /// needs to be accessed. Calling Merge on a DB without a merge operator
    /// would result in Status::NotSupported. The client must ensure that the
    /// merge operator supplied here has the same name and *exactly* the same
    /// semantics as the merge operator provided to previous open calls on
    /// the same DB. The only exception is reserved for upgrade, where a DB
    /// previously without a merge operator is introduced to Merge operation
    /// for the first time. It's necessary to specify a merge operator when
    /// openning the DB in this case.
    ///
    /// Default: nullptr
    pub fn merge_operator(self, val: Box<MergeOperator>) -> Self {
        unsafe {
            let raw_ptr = Box::into_raw(Box::new(val)); // Box<Box<MergeOperator>>
            ll::rocks_cfoptions_set_merge_operator_by_merge_op_trait(self.raw, raw_ptr as *mut _);
        }
        self
    }

    pub fn associative_merge_operator(self, val: Box<AssociativeMergeOperator>) -> Self {
        unsafe {
            // FIXME: into_raw
            let raw_ptr = Box::into_raw(Box::new(val)); // Box<Box<AssociativeMergeOperator>>
            ll::rocks_cfoptions_set_merge_operator_by_assoc_op_trait(self.raw, raw_ptr as *mut _);
        }
        self
    }

    /// A single CompactionFilter instance to call into during compaction.
    /// Allows an application to modify/delete a key-value during background
    /// compaction.
    ///
    /// If the client requires a new compaction filter to be used for different
    /// compaction runs, it can specify compaction_filter_factory instead of this
    /// option.  The client should specify only one of the two.
    /// compaction_filter takes precedence over compaction_filter_factory if
    /// client specifies both.
    ///
    /// If multithreaded compaction is being used, the supplied CompactionFilter
    /// instance may be used from different threads concurrently and so should be
    /// thread-safe.
    ///
    /// Default: nullptr
    pub fn compaction_filter(self, filter: Box<CompactionFilter + Sync>) -> Self {
        unsafe {
            // FIXME: mem leaks
            // CFOptions.compaction_filter is a raw pointer
            let raw_ptr = Box::into_raw(Box::new(filter)); // Box<Box<CompactionFilter>>
            ll::rocks_cfoptions_set_compaction_filter_by_trait(self.raw, raw_ptr as *mut _);
        }
        self
    }

    /// This is a factory that provides compaction filter objects which allow
    /// an application to modify/delete a key-value during background compaction.
    ///
    /// A new filter will be created on each compaction run.  If multithreaded
    /// compaction is being used, each created CompactionFilter will only be used
    /// from a single thread and so does not need to be thread-safe.
    ///
    /// Default: nullptr
    pub fn compaction_filter_factory(self, factory: Box<CompactionFilterFactory>) -> Self {
        // unsafe {
        // ll::rocks_cfoptions_set_compaction_filter_factory(self.raw, )
        // }
        // self
        unimplemented!()
    }

    // -------------------
    // Parameters that affect performance
    // -------------------

    /// Amount of data to build up in memory (backed by an unsorted log
    /// on disk) before converting to a sorted on-disk file.
    ///
    /// Larger values increase performance, especially during bulk loads.
    /// Up to max_write_buffer_number write buffers may be held in memory
    /// at the same time,
    /// so you may wish to adjust this parameter to control memory usage.
    /// Also, a larger write buffer will result in a longer recovery time
    /// the next time the database is opened.
    ///
    /// Note that write_buffer_size is enforced per column family.
    /// See db_write_buffer_size for sharing memory across column families.
    ///
    /// Default: 64MB
    ///
    /// Dynamically changeable through `SetOptions()` API
    pub fn write_buffer_size(self, val: usize) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_write_buffer_size(self.raw, val);
        }
        self
    }

    /// Compress blocks using the specified compression algorithm.  This
    /// parameter can be changed dynamically.
    ///
    /// Default: kSnappyCompression, if it's supported. If snappy is not linked
    /// with the library, the default is kNoCompression.
    ///
    /// Typical speeds of kSnappyCompression on an Intel(R) Core(TM)2 2.4GHz:
    ///
    /// - ~200-500MB/s compression
    /// - ~400-800MB/s decompression
    ///
    /// Note that these speeds are significantly faster than most
    /// persistent storage speeds, and therefore it is typically never
    /// worth switching to kNoCompression.  Even if the input data is
    /// incompressible, the kSnappyCompression implementation will
    /// efficiently detect that and will switch to uncompressed mode.
    pub fn compression(self, val: CompressionType) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_compression(self.raw, mem::transmute(val));
        }
        self
    }

    /// Compression algorithm that will be used for the bottommost level that
    /// contain files. If level-compaction is used, this option will only affect
    /// levels after base level.
    ///
    /// Default: kDisableCompressionOption (Disabled)
    pub fn bottommost_compression(self, val: CompressionType) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_bottommost_compression(self.raw, mem::transmute(val));
        }
        self
    }

    /// different options for compression algorithms
    pub fn compression_opts(self, val: CompressionOptions) -> Self {
        unsafe {
            // FIXME: name changes from opts to options
            ll::rocks_cfoptions_set_compression_options(
                self.raw,
                val.window_bits,
                val.level,
                val.strategy,
                val.max_dict_bytes,
            );
        }
        self
    }

    /// Number of files to trigger level-0 compaction. A value <0 means that
    /// level-0 compaction will not be triggered by number of files at all.
    ///
    /// Default: 4
    ///
    /// Dynamically changeable through `SetOptions()` API
    pub fn level0_file_num_compaction_trigger(self, val: i32) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_level0_file_num_compaction_trigger(self.raw, val);
        }
        self
    }

    /// If non-nullptr, use the specified function to determine the
    /// prefixes for keys.  These prefixes will be placed in the filter.
    /// Depending on the workload, this can reduce the number of read-IOP
    /// cost for scans when a prefix is passed via ReadOptions to
    /// db.NewIterator().  For prefix filtering to work properly,
    /// "prefix_extractor" and "comparator" must be such that the following
    /// properties hold:
    ///
    /// 1) key.starts_with(prefix(key))
    /// 2) Compare(prefix(key), key) <= 0.
    /// 3) If Compare(k1, k2) <= 0, then Compare(prefix(k1), prefix(k2)) <= 0
    /// 4) prefix(prefix(key)) == prefix(key)
    ///
    /// Default: nullptr
    // FIXME: split other prefix extractor variants
    pub fn prefix_extractor(self, val: Box<SliceTransform + Sync>) -> Self {
        unsafe {
            let raw_ptr = Box::into_raw(Box::new(val));
            ll::rocks_cfoptions_set_prefix_extractor_by_trait(self.raw, raw_ptr as *mut _);
        }
        self
    }

    pub fn prefix_extractor_fixed(self, len: usize) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_prefix_extractor_fixed_prefix(self.raw, len);
        }
        self
    }
    pub fn prefix_extractor_capped(self, len: usize) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_prefix_extractor_capped_prefix(self.raw, len);
        }
        self
    }
    pub fn prefix_extractor_noop(self) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_prefix_extractor_noop(self.raw);
        }
        self
    }

    /// Control maximum total data size for a level.
    /// max_bytes_for_level_base is the max total for level-1.
    /// Maximum number of bytes for level L can be calculated as
    /// (max_bytes_for_level_base) * (max_bytes_for_level_multiplier ^ (L-1))
    /// For example, if max_bytes_for_level_base is 200MB, and if
    /// max_bytes_for_level_multiplier is 10, total data size for level-1
    /// will be 200MB, total file size for level-2 will be 2GB,
    /// and total file size for level-3 will be 20GB.
    ///
    /// Default: 256MB.
    ///
    /// Dynamically changeable through `SetOptions()` API
    pub fn max_bytes_for_level_base(self, val: u64) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_max_bytes_for_level_base(self.raw, val);
        }
        self
    }

    /// Disable automatic compactions. Manual compactions can still
    /// be issued on this column family
    ///
    /// Dynamically changeable through `SetOptions()` API
    pub fn disable_auto_compactions(self, val: bool) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_disable_auto_compactions(self.raw, val as u8);
        }
        self
    }

    /// This is a factory that provides TableFactory objects.
    ///
    /// Default: a block-based table factory that provides a default
    /// implementation of TableBuilder and TableReader with default
    /// BlockBasedTableOptions.
    ///
    /// For Rust: use 3 different function
    pub fn table_factory_plain(self, opt: PlainTableOptions) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_plain_table_factory(self.raw, opt.raw());
        }
        self
    }

    pub fn table_factory_block_based(self, opt: BlockBasedTableOptions) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_block_based_table_factory(self.raw, opt.raw());
        }
        self
    }

    pub fn table_factory_cuckoo(self, opt: CuckooTableOptions) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_cuckoo_table_factory(self.raw, opt.raw());
        }
        self
    }

    // pub fn table_factory(self, val: ()) -> Self {
    // panic!("use any of plain_table_factory, block_based_table_factory and cuckoo_table_factory")
    // }
    //

    // Following: AdvancedColumnFamilyOptions

    /// The maximum number of write buffers that are built up in memory.
    /// The default and the minimum number is 2, so that when 1 write buffer
    /// is being flushed to storage, new writes can continue to the other
    /// write buffer.
    ///
    /// If `max_write_buffer_number` > 3, writing will be slowed down to
    /// `options.delayed_write_rate` if we are writing to the last write buffer
    /// allowed.
    ///
    /// Default: 2
    ///
    /// Dynamically changeable through `SetOptions()` API
    pub fn max_write_buffer_number(self, val: i32) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_max_write_buffer_number(self.raw, val);
        }
        self
    }

    /// The minimum number of write buffers that will be merged together
    /// before writing to storage.  If set to 1, then
    /// all write buffers are flushed to L0 as individual files and this increases
    /// read amplification because a get request has to check in all of these
    /// files. Also, an in-memory merge may result in writing lesser
    /// data to storage if there are duplicate records in each of these
    /// individual write buffers.
    ///
    /// Default: 1
    pub fn min_write_buffer_number_to_merge(self, val: i32) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_min_write_buffer_number_to_merge(self.raw, val);
        }
        self
    }

    /// The total maximum number of write buffers to maintain in memory including
    /// copies of buffers that have already been flushed.  Unlike
    /// max_write_buffer_number, this parameter does not affect flushing.
    /// This controls the minimum amount of write history that will be available
    /// in memory for conflict checking when Transactions are used.
    ///
    /// When using an OptimisticTransactionDB:
    ///
    /// If this value is too low, some transactions may fail at commit time due
    /// to not being able to determine whether there were any write conflicts.
    ///
    /// When using a TransactionDB:
    ///
    /// If Transaction::SetSnapshot is used, TransactionDB will read either
    /// in-memory write buffers or SST files to do write-conflict checking.
    /// Increasing this value can reduce the number of reads to SST files
    /// done for conflict detection.
    ///
    /// Setting this value to 0 will cause write buffers to be freed immediately
    /// after they are flushed.
    ///
    /// If this value is set to -1, 'max_write_buffer_number' will be used.
    ///
    /// Default:
    ///
    /// If using a TransactionDB/OptimisticTransactionDB, the default value will
    /// be set to the value of 'max_write_buffer_number' if it is not explicitly
    /// set by the user.  Otherwise, the default is 0.
    pub fn max_write_buffer_number_to_maintain(self, val: i32) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_max_write_buffer_number_to_maintain(self.raw, val);
        }
        self
    }

    /// Allows thread-safe inplace updates. If this is true, there is no way to
    /// achieve point-in-time consistency using snapshot or iterator (assuming
    /// concurrent updates). Hence iterator and multi-get will return results
    /// which are not consistent as of any point-in-time.
    ///
    /// If inplace_callback function is not set,
    /// Put(key, new_value) will update inplace the existing_value iff
    ///
    /// * key exists in current memtable
    /// * new sizeof(new_value) <= sizeof(existing_value)
    /// * existing_value for that key is a put i.e. kTypeValue
    ///
    /// If inplace_callback function is set, check doc for inplace_callback.
    ///
    /// Default: false.
    pub fn inplace_update_support(self, val: bool) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_inplace_update_support(self.raw, val as u8);
        }
        self
    }

    /// Number of locks used for inplace update
    ///
    /// Default: 10000, if inplace_update_support = true, else 0.
    ///
    /// Dynamically changeable through `SetOptions()` API
    pub fn inplace_update_num_locks(self, val: usize) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_inplace_update_num_locks(self.raw, val);
        }
        self
    }

    /// * existing_value - pointer to previous value (from both memtable and sst).
    ///                  pub nullptr if key doesn't exist
    /// * existing_value_size - pointer to size of existing_value).
    ///                       pub nullptr if key doesn't exist
    /// * delta_value - Delta value to be merged with the existing_value.
    ///               pub Stored in transaction logs.
    /// * merged_value - Set when delta is applied on the previous value.
    ///
    /// Applicable only when inplace_update_support is true,
    /// this callback function is called at the time of updating the memtable
    /// as part of a Put operation, lets say Put(key, delta_value). It allows the
    /// 'delta_value' specified as part of the Put operation to be merged with
    /// an 'existing_value' of the key in the database.
    ///
    /// If the merged value is smaller in size that the 'existing_value',
    /// then this function can update the 'existing_value' buffer inplace and
    /// the corresponding 'existing_value'_size pointer, if it wishes to.
    /// The callback should return UpdateStatus::UPDATED_INPLACE.
    /// In this case. (In this case, the snapshot-semantics of the rocksdb
    /// Iterator is not atomic anymore).
    ///
    /// If the merged value is larger in size than the 'existing_value' or the
    /// application does not wish to modify the 'existing_value' buffer inplace,
    /// then the merged value should be returned via *merge_value. It is set by
    /// merging the 'existing_value' and the Put 'delta_value'. The callback should
    /// return UpdateStatus::UPDATED in this case. This merged value will be added
    /// to the memtable.
    ///
    /// If merging fails or the application does not wish to take any action,
    /// then the callback should return `UpdateStatus::UPDATE_FAILED`.
    ///
    /// Please remember that the original call from the application is Put(key,
    /// delta_value). So the transaction log (if enabled) will still contain (key,
    /// delta_value). The 'merged_value' is not stored in the transaction log.
    /// Hence the inplace_callback function should be consistent across db reopens.
    ///
    /// Default: nullptr
    ///
    /// Rust: TODO: unimplemented!()
    pub fn inplace_callback<F>(self, val: Option<()>) -> Self {
        //     unsafe {
        //          ll::rocks_cfoptions_set_inplace_callback(self.raw, val);
        //     }
        //     self
        unimplemented!()
    }

    // UpdateStatus (*inplace_callback)(char* existing_value,
    // uint32_t* existing_value_size,
    // Slice delta_value,
    // std::string* merged_value) = nullptr;

    /// if prefix_extractor is set and memtable_prefix_bloom_size_ratio is not 0,
    /// create prefix bloom for memtable with the size of
    /// write_buffer_size * memtable_prefix_bloom_size_ratio.
    /// If it is larger than 0.25, it is santinized to 0.25.
    ///
    /// Default: 0 (disable)
    ///
    /// Dynamically changeable through `SetOptions()` API
    pub fn memtable_prefix_bloom_size_ratio(self, val: f64) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_memtable_prefix_bloom_size_ratio(self.raw, val);
        }
        self
    }

    /// Page size for huge page for the arena used by the memtable. If <=0, it
    /// won't allocate from huge page but from malloc.
    /// Users are responsible to reserve huge pages for it to be allocated. For
    /// example:
    ///
    /// > `pub sysctl -w vm.nr_hugepages=20`
    ///
    /// See linux doc Documentation/vm/hugetlbpage.txt
    ///
    /// If there isn't enough free huge page available, it will fall back to
    /// malloc.
    ///
    /// Dynamically changeable through `SetOptions()` API
    pub fn memtable_huge_page_size(self, val: usize) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_memtable_huge_page_size(self.raw, val);
        }
        self
    }

    /// If non-nullptr, memtable will use the specified function to extract
    /// prefixes for keys, and for each prefix maintain a hint of insert location
    /// to reduce CPU usage for inserting keys with the prefix. Keys out of
    /// domain of the prefix extractor will be insert without using hints.
    ///
    /// Currently only the default skiplist based memtable implements the feature.
    /// All other memtable implementation will ignore the option. It incurs ~250
    /// additional bytes of memory overhead to store a hint for each prefix.
    /// Also concurrent writes (when allow_concurrent_memtable_write is true) will
    /// ignore the option.
    ///
    /// The option is best suited for workloads where keys will likely to insert
    /// to a location close the the last inserted key with the same prefix.
    /// One example could be inserting keys of the form (prefix + timestamp),
    /// and keys of the same prefix always comes in with time order. Another
    /// example would be updating the same key over and over again, in which case
    /// the prefix can be the key itself.
    ///
    /// Default: nullptr (disable)
    pub fn memtable_insert_with_hint_prefix_extractor(self, val: Box<SliceTransform + Sync>) -> Self {
        unsafe {
            let raw_ptr = Box::into_raw(Box::new(val));
            ll::rocks_cfoptions_set_memtable_insert_with_hint_prefix_extractor_by_trait(self.raw, raw_ptr as *mut _);
        }
        self
    }

    pub fn memtable_insert_with_hint_prefix_extractor_fixed(self, len: usize) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_memtable_insert_with_hint_prefix_extractor_fixed_prefix(self.raw, len);
        }
        self
    }
    pub fn memtable_insert_with_hint_prefix_extractor_capped(self, len: usize) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_memtable_insert_with_hint_prefix_extractor_capped_prefix(self.raw, len);
        }
        self
    }
    pub fn memtable_insert_with_hint_prefix_extractor_noop(self) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_memtable_insert_with_hint_prefix_extractor_noop(self.raw);
        }
        self
    }

    /// Control locality of bloom filter probes to improve cache miss rate.
    ///
    /// This option only applies to memtable prefix bloom and plaintable
    /// prefix bloom. It essentially limits every bloom checking to one cache line.
    /// This optimization is turned off when set to 0, and positive number to turn
    /// it on.
    ///
    /// Default: 0
    pub fn bloom_locality(self, val: u32) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_bloom_locality(self.raw, val);
        }
        self
    }

    /// size of one block in arena memory allocation.
    ///
    /// If <= 0, a proper value is automatically calculated (usually 1/8 of
    /// writer_buffer_size, rounded up to a multiple of 4KB).
    ///
    /// There are two additional restriction of the The specified size:
    ///
    /// 1. size should be in the range of [4096, 2 << 30] and
    /// 2. be the multiple of the CPU word (which helps with the memory
    ///     alignment).
    ///
    /// We'll automatically check and adjust the size number to make sure it
    /// conforms to the restrictions.
    ///
    /// Default: 0
    ///
    /// Dynamically changeable through `SetOptions()` API
    pub fn arena_block_size(self, val: usize) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_arena_block_size(self.raw, val);
        }
        self
    }


    /// Different levels can have different compression policies. There
    /// are cases where most lower levels would like to use quick compression
    /// algorithms while the higher levels (which have more data) use
    /// compression algorithms that have better compression but could
    /// be slower. This array, if non-empty, should have an entry for
    /// each level of the database; these override the value specified in
    /// the previous field 'compression'.
    ///
    /// NOTICE if level_compaction_dynamic_level_bytes=true,
    /// compression_per_level[0] still determines L0, but other elements
    /// of the array are based on base level (the level L0 files are merged
    /// to), and may not match the level users see from info log for metadata.
    /// If L0 files are merged to level-n, then, for i>0, compression_per_level[i]
    /// determines compaction type for level n+i-1.
    ///
    /// For example, if we have three 5 levels, and we determine to merge L0
    /// data to L4 (which means L1..L3 will be empty), then the new files go to
    /// L4 uses compression type compression_per_level[1].
    ///
    /// If now L0 is merged to L2. Data goes to L2 will be compressed
    /// according to compression_per_level[1], L3 using compression_per_level[2]
    /// and L4 using compression_per_level[3]. Compaction for each level can
    /// change when data grows.
    pub fn compression_per_level(self, val: &[CompressionType]) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_compression_per_level(
                self.raw,
                mem::transmute(val.as_ptr()), // repr(C)
                val.len(),
            );
        }
        self
    }

    /// Number of levels for this database
    ///
    /// Default: 7
    pub fn num_levels(self, val: i32) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_num_levels(self.raw, val);
        }
        self
    }

    /// Soft limit on number of level-0 files. We start slowing down writes at this
    /// point. A value <0 means that no writing slow down will be triggered by
    /// number of files in level-0.
    ///
    /// Default: 20
    ///
    /// Dynamically changeable through `SetOptions()` API
    pub fn level0_slowdown_writes_trigger(self, val: i32) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_level0_slowdown_writes_trigger(self.raw, val);
        }
        self
    }

    /// Maximum number of level-0 files.  We stop writes at this point.
    ///
    /// Default: 36
    ///
    /// Dynamically changeable through `SetOptions()` API
    pub fn level0_stop_writes_trigger(self, val: i32) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_level0_stop_writes_trigger(self.raw, val);
        }
        self
    }

    /// Target file size for compaction.
    ///
    /// target_file_size_base is per-file size for level-1.
    /// Target file size for level L can be calculated by
    /// target_file_size_base * (target_file_size_multiplier ^ (L-1))
    /// For example, if target_file_size_base is 2MB and
    /// target_file_size_multiplier is 10, then each file on level-1 will
    /// be 2MB, and each file on level 2 will be 20MB,
    /// and each file on level-3 will be 200MB.
    ///
    /// Default: 64MB.
    ///
    /// Dynamically changeable through `SetOptions()` API
    pub fn target_file_size_base(self, val: u64) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_target_file_size_base(self.raw, val);
        }
        self
    }

    /// By default `target_file_size_multiplier` is 1, which means
    /// by default files in different levels will have similar size.
    ///
    /// Dynamically changeable through `SetOptions()` API
    pub fn target_file_size_multiplier(self, val: i32) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_target_file_size_multiplier(self.raw, val);
        }
        self
    }

    /// If true, RocksDB will pick target size of each level dynamically.
    /// We will pick a base level b >= 1. L0 will be directly merged into level b,
    /// instead of always into level 1. Level 1 to b-1 need to be empty.
    /// We try to pick b and its target size so that
    ///
    /// 1. target size is in the range of
    ///    (max_bytes_for_level_base / max_bytes_for_level_multiplier,
    ///     max_bytes_for_level_base]
    /// 2. target size of the last level (level num_levels-1) equals to extra size
    ///    of the level.
    ///
    /// At the same time max_bytes_for_level_multiplier and
    /// max_bytes_for_level_multiplier_additional are still satisfied.
    ///
    /// With this option on, from an empty DB, we make last level the base level,
    /// which means merging L0 data into the last level, until it exceeds
    /// max_bytes_for_level_base. And then we make the second last level to be
    /// base level, to start to merge L0 data to second last level, with its
    /// target size to be 1/max_bytes_for_level_multiplier of the last level's
    /// extra size. After the data accumulates more so that we need to move the
    /// base level to the third last one, and so on.
    ///
    /// For example, assume max_bytes_for_level_multiplier=10, num_levels=6,
    /// and max_bytes_for_level_base=10MB.
    ///
    /// Target sizes of level 1 to 5 starts with:
    ///
    /// > `[- - - - 10MB]`
    ///
    /// with base level is level. Target sizes of level 1 to 4 are not applicable
    /// because they will not be used.
    ///
    /// Until the size of Level 5 grows to more than 10MB, say 11MB, we make
    /// base target to level 4 and now the targets looks like:
    ///
    /// > `[- - - 1.1MB 11MB]`
    ///
    /// While data are accumulated, size targets are tuned based on actual data
    /// of level 5. When level 5 has 50MB of data, the target is like:
    ///
    /// > `[- - - 5MB 50MB]`
    ///
    /// Until level 5's actual size is more than 100MB, say 101MB. Now if we keep
    /// level 4 to be the base level, its target size needs to be 10.1MB, which
    /// doesn't satisfy the target size range. So now we make level 3 the target
    /// size and the target sizes of the levels look like:
    ///
    /// > `[- - 1.01MB 10.1MB 101MB]`
    ///
    /// In the same way, while level 5 further grows, all levels' targets grow,
    /// like
    ///
    /// > `[- - 5MB 50MB 500MB]`
    ///
    /// Until level 5 exceeds 1000MB and becomes 1001MB, we make level 2 the
    /// base level and make levels' target sizes like this:
    ///
    /// > `[- 1.001MB 10.01MB 100.1MB 1001MB]`
    ///
    /// and go on...
    ///
    /// By doing it, we give max_bytes_for_level_multiplier a priority against
    /// max_bytes_for_level_base, for a more predictable LSM tree shape. It is
    /// useful to limit worse case space amplification.
    ///
    /// `max_bytes_for_level_multiplier_additional` is ignored with this flag on.
    ///
    /// Turning this feature on or off for an existing DB can cause unexpected
    /// LSM tree structure so it's not recommended.
    ///
    /// NOTE: this option is experimental
    ///
    /// Default: false
    pub fn level_compaction_dynamic_level_bytes(self, val: bool) -> Self {
        unsafe {
            // wtf this name is a bool?
            ll::rocks_cfoptions_set_level_compaction_dynamic_level_bytes(self.raw, val as u8);
        }
        self
    }

    /// Default: 10.
    ///
    /// Dynamically changeable through `SetOptions()` API
    pub fn max_bytes_for_level_multiplier(self, val: f64) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_max_bytes_for_level_multiplier(self.raw, val);
        }
        self
    }

    /// Different max-size multipliers for different levels.
    ///
    /// These are multiplied by max_bytes_for_level_multiplier to arrive
    /// at the max-size of each level.
    ///
    /// Default: 1
    ///
    /// Dynamically changeable through `SetOptions()` API
    pub fn max_bytes_for_level_multiplier_additional(self, val: &[i32]) -> Self {
        let cval = val.iter().map(|&v| v as c_int).collect::<Vec<_>>();
        let num_levels = val.len();
        unsafe {

            ll::rocks_cfoptions_set_max_bytes_for_level_multiplier_additional(self.raw, cval.as_ptr(), num_levels);
        }
        self
    }

    /// We try to limit number of bytes in one compaction to be lower than this
    /// threshold. But it's not guaranteed.
    /// Value 0 will be sanitized.
    ///
    /// Default: result.target_file_size_base * 25
    pub fn max_compaction_bytes(self, val: u64) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_max_compaction_bytes(self.raw, val);
        }
        self
    }

    /// All writes will be slowed down to at least delayed_write_rate if estimated
    /// bytes needed to be compaction exceed this threshold.
    ///
    /// Default: 64GB
    pub fn soft_pending_compaction_bytes_limit(self, val: u64) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_soft_pending_compaction_bytes_limit(self.raw, val);
        }
        self
    }

    /// All writes are stopped if estimated bytes needed to be compaction exceed
    /// this threshold.
    ///
    /// Default: 256GB
    pub fn hard_pending_compaction_bytes_limit(self, val: u64) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_hard_pending_compaction_bytes_limit(self.raw, val);
        }
        self
    }

    /// The compaction style.
    ///
    /// Default: CompactionStyleLevel
    pub fn compaction_style(self, val: CompactionStyle) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_compaction_style(self.raw, mem::transmute(val));
        }
        self
    }

    /// If level compaction_style = kCompactionStyleLevel, for each level,
    /// which files are prioritized to be picked to compact.
    ///
    /// Default: ByCompensatedSize
    pub fn compaction_pri(self, val: CompactionPri) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_compaction_pri(self.raw, mem::transmute(val));
        }
        self
    }

    /// The options needed to support Universal Style compactions
    pub fn compaction_options_universal(self, opt: CompactionOptionsUniversal) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_universal_compaction_options(self.raw, opt.raw());
        }
        self
    }

    /// The options for FIFO compaction style
    pub fn compaction_options_fifo(self, val: CompactionOptionsFIFO) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_fifo_compaction_options(self.raw, val.raw());
        }
        self
    }

    /// An iteration->Next() sequentially skips over keys with the same
    /// user-key unless this option is set. This number specifies the number
    /// of keys (with the same userkey) that will be sequentially
    /// skipped before a reseek is issued.
    ///
    /// Default: 8
    ///
    /// Dynamically changeable through `SetOptions()` API
    pub fn max_sequential_skip_in_iterations(self, val: u64) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_max_sequential_skip_in_iterations(self.raw, val);
        }
        self
    }

    /// This creates MemTableReps that are backed by an std::vector. On iteration,
    /// the vector is sorted. This is useful for workloads where iteration is very
    /// rare and writes are generally not issued after reads begin.
    ///
    /// # Arguments
    ///
    /// - count: Passed to the constructor of the underlying std::vector of each
    ///   VectorRep. On initialization, the underlying array will be at least count
    ///   bytes reserved for usage.
    ///
    ///   Default: 0
    pub fn memtable_factory_vector_rep(self, count: usize) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_memtable_vector_rep(self.raw, count);
        }
        self
    }

    /// This class contains a fixed array of buckets, each
    /// pointing to a skiplist (null if the bucket is empty).
    ///
    /// # Arguments
    ///
    /// - bucket_count: number of fixed array buckets
    ///
    ///   Default: 1000000
    /// - skiplist_height: the max height of the skiplist
    ///
    ///   Default: 4
    /// - skiplist_branching_factor: probabilistic size ratio between adjacent
    ///   link lists in the skiplist
    ///
    ///   Default: 4
    pub fn memtable_factory_hash_skip_list_rep(
        self,
        bucket_count: usize,
        skiplist_height: i32,
        skiplist_branching_factor: i32,
    ) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_hash_skip_list_rep(
                self.raw,
                bucket_count,
                skiplist_height,
                skiplist_branching_factor,
            );
        }
        self
    }

    /// The factory is to create memtables based on a hash table:
    /// it contains a fixed array of buckets, each pointing to either a linked list
    /// or a skip list if number of entries inside the bucket exceeds
    /// threshold_use_skiplist.
    ///
    /// # Arguments
    ///
    /// - bucket_count: number of fixed array buckets
    ///
    ///   Default: 50000
    /// - huge_page_tlb_size: if <=0, allocate the hash table bytes from malloc.
    ///   Otherwise from huge page TLB. The user needs to reserve
    ///   huge pages for it to be allocated, like:
    ///
    ///   > sysctl -w vm.nr_hugepages=20
    ///
    ///   See linux doc Documentation/vm/hugetlbpage.txt
    ///
    ///   Default: 0
    /// - bucket_entries_logging_threshold: if number of entries in one bucket
    ///   exceeds this number, log about it.
    ///
    ///   Default: 4096
    /// - if_log_bucket_dist_when_flash: if true, log distribution of number of
    ///   entries when flushing.
    ///
    ///   Default: true
    /// - threshold_use_skiplist: a bucket switches to skip list if number of
    ///   entries exceed this parameter.
    ///
    ///   Default: 256
    pub fn memtable_factory_hash_link_list_rep(self, bucket_count: usize) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_hash_link_list_rep(self.raw, bucket_count);
        }
        self
    }

    /// This factory creates a cuckoo-hashing based mem-table representation.
    /// Cuckoo-hash is a closed-hash strategy, in which all key/value pairs
    /// are stored in the bucket array itself intead of in some data structures
    /// external to the bucket array.  In addition, each key in cuckoo hash
    /// has a constant number of possible buckets in the bucket array.  These
    /// two properties together makes cuckoo hash more memory efficient and
    /// a constant worst-case read time.  Cuckoo hash is best suitable for
    /// point-lookup workload.
    ///
    /// When inserting a key / value, it first checks whether one of its possible
    /// buckets is empty.  If so, the key / value will be inserted to that vacant
    /// bucket.  Otherwise, one of the keys originally stored in one of these
    /// possible buckets will be "kicked out" and move to one of its possible
    /// buckets (and possibly kicks out another victim.)  In the current
    /// implementation, such "kick-out" path is bounded.  If it cannot find a
    /// "kick-out" path for a specific key, this key will be stored in a backup
    /// structure, and the current memtable to be forced to immutable.
    ///
    /// Note that currently this mem-table representation does not support
    /// snapshot (i.e., it only queries latest state) and iterators.  In addition,
    /// MultiGet operation might also lose its atomicity due to the lack of
    /// snapshot support.
    ///
    /// # Arguments
    ///
    /// - write_buffer_size: the write buffer size in bytes.
    /// - average_data_size: the average size of key + value in bytes.  This value
    ///   together with write_buffer_size will be used to compute the number
    ///   of buckets.
    ///
    ///   Default: 64
    /// - hash_function_count: the number of hash functions that will be used by
    ///   the cuckoo-hash.  The number also equals to the number of possible
    ///   buckets each key will have.
    ///
    ///   Default: 4
    pub fn memtable_factory_hash_cuckoo_rep(
        self,
        write_buffer_size: usize,
        average_data_size: usize,
        hash_function_count: u32,
    ) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_hash_cuckoo_rep(
                self.raw,
                write_buffer_size,
                average_data_size,
                hash_function_count,
            );
        }
        self
    }

    /// Block-based table related options are moved to BlockBasedTableOptions.
    /// Related options that were originally here but now moved include:
    ///
    /// * no_block_cache
    /// * block_cache
    /// * block_cache_compressed
    /// * block_size
    /// * block_size_deviation
    /// * block_restart_interval
    /// * filter_policy
    /// * whole_key_filtering
    ///
    /// If you'd like to customize some of these options, you will need to
    /// use NewBlockBasedTableFactory() to construct a new table factory.
    ///
    /// This option allows user to collect their own interested statistics of
    /// the tables.
    ///
    /// Default: empty vector -- no user-defined statistics collection will be
    /// performed.
    ///
    /// Rust: add one at a time
    pub fn table_properties_collector_factory(self, val: Box<TablePropertiesCollectorFactory>) -> Self {
        unsafe {
            let raw_ptr = Box::into_raw(Box::new(val));
            ll::rocks_cfoptions_add_table_properties_collector_factories_by_trait(self.raw, raw_ptr as *mut _);
        }
        self
    }

    /// Maximum number of successive merge operations on a key in the memtable.
    ///
    /// When a merge operation is added to the memtable and the maximum number of
    /// successive merges is reached, the value of the key will be calculated and
    /// inserted into the memtable instead of the merge operation. This will
    /// ensure that there are never more than max_successive_merges merge
    /// operations in the memtable.
    ///
    /// Default: 0 (disabled)
    ///
    /// Dynamically changeable through `SetOptions()` API
    pub fn max_successive_merges(self, val: usize) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_max_successive_merges(self.raw, val);
        }
        self
    }

    /// This flag specifies that the implementation should optimize the filters
    /// mainly for cases where keys are found rather than also optimize for keys
    /// missed. This would be used in cases where the application knows that
    /// there are very few misses or the performance in the case of misses is not
    /// important.
    ///
    /// For now, this flag allows us to not store filters for the last level i.e
    /// the largest level which contains data of the LSM store. For keys which
    /// are hits, the filters in this level are not useful because we will search
    /// for the data anyway. NOTE: the filters in other levels are still useful
    /// even for key hit because they tell us whether to look in that level or go
    /// to the higher level.
    ///
    /// Default: false
    pub fn optimize_filters_for_hits(self, val: bool) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_optimize_filters_for_hits(self.raw, val as u8);
        }
        self
    }

    /// After writing every SST file, reopen it and read all the keys.
    ///
    /// Default: false
    pub fn paranoid_file_checks(self, val: bool) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_paranoid_file_checks(self.raw, val as u8);
        }
        self
    }

    /// In debug mode, RocksDB run consistency checks on the LSM everytime the LSM
    /// change (Flush, Compaction, AddFile). These checks are disabled in release
    /// mode, use this option to enable them in release mode as well.
    ///
    /// Default: false
    pub fn force_consistency_checks(self, val: bool) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_force_consistency_checks(self.raw, val as u8);
        }
        self
    }

    /// Measure IO stats in compactions and flushes, if true.
    ///
    /// Default: false
    pub fn report_bg_io_stats(self, val: bool) -> Self {
        unsafe {
            ll::rocks_cfoptions_set_report_bg_io_stats(self.raw, val as u8);
        }
        self
    }

    pub fn dump(&self, log: &mut Logger) {
        unimplemented!()
    }
}

/// Specify the file access pattern once a compaction is started.
/// It will be applied to all input files of a compaction.
///
/// Default: NORMAL
#[repr(C)]
pub enum AccessHint {
    None,
    Normal,
    Sequential,
    WillNeed,
}

/// Options for the DB
pub struct DBOptions {
    raw: *mut ll::rocks_dboptions_t,
}

impl Default for DBOptions {
    fn default() -> Self {
        DBOptions { raw: unsafe { ll::rocks_dboptions_create() } }
    }
}

impl Drop for DBOptions {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_dboptions_destroy(self.raw);
        }
    }
}

impl ToRaw<ll::rocks_dboptions_t> for DBOptions {
    fn raw(&self) -> *mut ll::rocks_dboptions_t {
        self.raw
    }
}

impl fmt::Display for DBOptions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            let cxx_string = ll::rocks_get_string_from_dboptions(self.raw);
            let len = ll::cxx_string_size(cxx_string);
            let base = ll::cxx_string_data(cxx_string);
            if !cxx_string.is_null() {
                let str_rep = str::from_utf8_unchecked(slice::from_raw_parts(base as *const u8, len));
                try!(f.write_str(str_rep));
                ll::cxx_string_destroy(cxx_string);
                Ok(())
            } else {
                f.write_str("DBOptions { error while converting to String }")
            }
        }
    }
}

impl DBOptions {
    unsafe fn from_ll(raw: *mut ll::rocks_dboptions_t) -> DBOptions {
        DBOptions { raw: raw }
    }

    /// If true, the database will be created if it is missing.
    ///
    /// Default: false
    pub fn create_if_missing(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_create_if_missing(self.raw, val as u8);
        }
        self
    }

    /// If true, missing column families will be automatically created.
    ///
    /// Default: false
    pub fn create_missing_column_families(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_create_missing_column_families(self.raw, val as u8);
        }
        self
    }

    /// If true, an error is raised if the database already exists.
    ///
    /// Default: false
    pub fn error_if_exists(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_error_if_exists(self.raw, val as u8);
        }
        self
    }

    /// If true, RocksDB will aggressively check consistency of the data.
    /// Also, if any of the  writes to the database fails (Put, Delete, Merge,
    /// Write), the database will switch to read-only mode and fail all other
    /// Write operations.
    ///
    /// In most cases you want this to be set to true.
    ///
    /// Default: true
    pub fn paranoid_checks(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_paranoid_checks(self.raw, val as u8);
        }
        self
    }

    /// Use the specified object to interact with the environment,
    /// e.g. to read/write files, schedule background work, etc.
    ///
    /// Default: Env::Default()
    pub fn env(self, env: &'static Env) -> Self {
        unsafe {
            ll::rocks_dboptions_set_env(self.raw, env.raw());
        }
        self
    }

    /// Use to control write rate of flush and compaction. Flush has higher
    /// priority than compaction. Rate limiting is disabled if nullptr.
    /// If rate limiter is enabled, bytes_per_sync is set to 1MB by default.
    ///
    /// Default: nullptr
    pub fn rate_limiter(self, val: Option<RateLimiter>) -> Self {
        unsafe {
            if let Some(limiter) = val {
                ll::rocks_dboptions_set_ratelimiter(self.raw, limiter.raw());
            } else {
                ll::rocks_dboptions_set_ratelimiter(self.raw, ptr::null_mut());
            }
        }
        self
    }

    /// Use to track SST files and control their file deletion rate.
    ///
    /// Features:
    ///
    ///  - Throttle the deletion rate of the SST files.
    ///  - Keep track the total size of all SST files.
    ///  - Set a maximum allowed space limit for SST files that when reached
    ///    the DB wont do any further flushes or compactions and will set the
    ///    background error.
    ///  - Can be shared between multiple dbs.
    ///
    /// Limitations:
    ///
    ///  - Only track and throttle deletes of SST files in
    ///    first db_path (db_name if db_paths is empty).
    ///
    /// Default: nullptr
    pub fn sst_file_manager(self, val: Option<SstFileManager>) -> Self {
        // unsafe {
        //     ll::rocks_dboptions_set_sst_file_manager(self.raw, val);
        // }
        // self
        unimplemented!()
    }

    /// Any internal progress/error information generated by the db will
    /// be written to info_log if it is non-nullptr, or to a file stored
    /// in the same directory as the DB contents if info_log is nullptr.
    ///
    /// Default: nullptr
    pub fn info_log(self, val: Option<Logger>) -> Self {
        unsafe {
            if let Some(logger) = val {
                ll::rocks_dboptions_set_info_log(self.raw, logger.raw());
            } else {
                ll::rocks_dboptions_set_info_log(self.raw, ptr::null_mut());
            }
        }
        self
    }

    pub fn info_log_level(self, val: InfoLogLevel) -> Self {
        unsafe {
            ll::rocks_dboptions_set_info_log_level(self.raw, mem::transmute(val));
        }
        self
    }

    /// Number of open files that can be used by the DB.  You may need to
    /// increase this if your database has a large working set. Value -1 means
    /// files opened are always kept open. You can estimate number of files based
    /// on target_file_size_base and target_file_size_multiplier for level-based
    /// compaction. For universal-style compaction, you can usually set it to -1.
    ///
    /// Default: -1
    pub fn max_open_files(self, val: i32) -> Self {
        unsafe {
            ll::rocks_dboptions_set_max_open_files(self.raw, val);
        }
        self
    }

    /// If max_open_files is -1, DB will open all files on DB::Open(). You can
    /// use this option to increase the number of threads used to open the files.
    ///
    /// Default: 16
    pub fn max_file_opening_threads(self, val: i32) -> Self {
        unsafe {
            ll::rocks_dboptions_set_max_file_opening_threads(self.raw, val);
        }
        self
    }

    /// Once write-ahead logs exceed this size, we will start forcing the flush of
    /// column families whose memtables are backed by the oldest live WAL file
    /// (i.e. the ones that are causing all the space amplification). If set to 0
    /// (default), we will dynamically choose the WAL size limit to be
    /// [sum of all write_buffer_size * max_write_buffer_number] * 4
    ///
    /// Default: 0
    pub fn max_total_wal_size(self, val: u64) -> Self {
        unsafe {
            ll::rocks_dboptions_set_max_total_wal_size(self.raw, val);
        }
        self
    }

    /// If non-null, then we should collect metrics about database operations
    pub fn statistics(self, val: Option<Statistics>) -> Self {
        match val {
            Some(stat) => unsafe { ll::rocks_dboptions_set_statistics(self.raw, stat.raw()) },
            None => unsafe { ll::rocks_dboptions_set_statistics(self.raw, ptr::null_mut()) },
        }
        self
    }

    /// If true, then every store to stable storage will issue a fsync.
    /// If false, then every store to stable storage will issue a fdatasync.
    /// This parameter should be set to true while storing data to
    /// filesystem like ext3 that can lose files after a reboot.
    ///
    /// Default: false
    ///
    /// Note: on many platforms fdatasync is defined as fsync, so this parameter
    /// would make no difference. Refer to fdatasync definition in this code base.
    pub fn use_fsync(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_use_fsync(self.raw, val as u8);
        }
        self
    }

    /// A list of paths where SST files can be put into, with its target size.
    /// Newer data is placed into paths specified earlier in the vector while
    /// older data gradually moves to paths specified later in the vector.
    ///
    /// For example, you have a flash device with 10GB allocated for the DB,
    /// as well as a hard drive of 2TB, you should config it to be:
    ///
    /// > [{"/flash_path", 10GB}, {"/hard_drive", 2TB}]
    ///
    /// The system will try to guarantee data under each path is close to but
    /// not larger than the target size. But current and future file sizes used
    /// by determining where to place a file are based on best-effort estimation,
    /// which means there is a chance that the actual size under the directory
    /// is slightly more than target size under some workloads. User should give
    /// some buffer room for those cases.
    ///
    /// If none of the paths has sufficient room to place a file, the file will
    /// be placed to the last path anyway, despite to the target size.
    ///
    /// Placing newer data to earlier paths is also best-efforts. User should
    /// expect user files to be placed in higher levels in some extreme cases.
    ///
    /// If left empty, only one path will be used, which is db_name passed when
    /// opening the DB.
    ///
    /// Default: empty
    pub fn db_paths<P: Into<DbPath>, T: IntoIterator<Item = P>>(self, val: T) -> Self {
        let paths = val.into_iter().map(|p| p.into()).collect::<Vec<_>>();
        let num_paths = paths.len();
        let mut cpaths = Vec::with_capacity(num_paths);
        let mut cpath_lens = Vec::with_capacity(num_paths);
        let mut sizes = Vec::with_capacity(num_paths);
        for dbpath in paths {
            cpaths.push(
                dbpath
                    .path
                    .to_str()
                    .map(|s| s.as_ptr() as _)
                    .unwrap_or_else(ptr::null),
            );
            cpath_lens.push(dbpath.path.to_str().map(|s| s.len()).unwrap_or_default());
            sizes.push(dbpath.target_size);
        }

        unsafe {
            ll::rocks_dboptions_set_db_paths(
                self.raw,
                cpaths.as_ptr(),
                cpath_lens.as_ptr(),
                sizes.as_ptr(),
                num_paths as c_int,
            );
        }
        self
    }

    /// This specifies the info LOG dir.
    ///
    /// If it is empty, the log files will be in the same dir as data.
    ///
    /// If it is non empty, the log files will be in the specified dir,
    /// and the db data dir's absolute path will be used as the log file
    /// name's prefix.
    pub fn db_log_dir<P: AsRef<Path>>(self, path: P) -> Self {
        unsafe {
            let path_str = path.as_ref().to_str().unwrap();
            ll::rocks_dboptions_set_db_log_dir(self.raw, path_str.as_ptr() as _, path_str.len());
        }
        self
    }

    /// This specifies the absolute dir path for write-ahead logs (WAL).
    ///
    /// If it is empty, the log files will be in the same dir as data,
    ///   dbname is used as the data dir by default
    ///
    /// If it is non empty, the log files will be in kept the specified dir.
    ///
    /// When destroying the db,
    /// all log files in wal_dir and the dir itself is deleted
    pub fn wal_dir<P: AsRef<Path>>(self, path: P) -> Self {
        unsafe {
            let path_str = path.as_ref().to_str().unwrap();
            ll::rocks_dboptions_set_wal_dir(self.raw, path_str.as_ptr() as _, path_str.len());
        }
        self
    }

    /// The periodicity when obsolete files get deleted. The default
    /// value is 6 hours. The files that get out of scope by compaction
    /// process will still get automatically delete on every compaction,
    /// regardless of this setting
    pub fn delete_obsolete_files_period_micros(self, val: u64) -> Self {
        unsafe {
            ll::rocks_dboptions_set_delete_obsolete_files_period_micros(self.raw, val);
        }
        self
    }

    /// Maximum number of concurrent background jobs (compactions and flushes).
    ///
    /// Default: 2
    pub fn max_background_jobs(self, val: i32) -> Self {
        unsafe {
            ll::rocks_dboptions_set_max_background_jobs(self.raw, val);
        }
        self
    }

    /// This value represents the maximum number of threads that will
    /// concurrently perform a compaction job by breaking it into multiple,
    /// smaller ones that are run simultaneously.
    ///
    /// Default: 1 (i.e. no subcompactions)
    pub fn max_subcompactions(self, val: u32) -> Self {
        unsafe {
            ll::rocks_dboptions_set_max_subcompactions(self.raw, val);
        }
        self
    }

    /// Specify the maximal size of the info log file. If the log file
    /// is larger than `max_log_file_size`, a new info log file will
    /// be created.
    ///
    /// If max_log_file_size == 0, all logs will be written to one
    /// log file.
    pub fn max_log_file_size(self, val: usize) -> Self {
        unsafe {
            ll::rocks_dboptions_set_max_log_file_size(self.raw, val);
        }
        self
    }

    /// Time for the info log file to roll (in seconds).
    /// If specified with non-zero value, log file will be rolled
    /// if it has been active longer than `log_file_time_to_roll`.
    ///
    /// Default: 0 (disabled)
    pub fn log_file_time_to_roll(self, val: usize) -> Self {
        unsafe {
            ll::rocks_dboptions_set_log_file_time_to_roll(self.raw, val);
        }
        self
    }

    /// Maximal info log files to be kept.
    ///
    /// Default: 1000
    pub fn keep_log_file_num(self, val: usize) -> Self {
        unsafe {
            ll::rocks_dboptions_set_keep_log_file_num(self.raw, val);
        }
        self
    }

    /// Recycle log files.
    ///
    /// If non-zero, we will reuse previously written log files for new
    /// logs, overwriting the old data.  The value indicates how many
    /// such files we will keep around at any point in time for later
    /// use.  This is more efficient because the blocks are already
    /// allocated and fdatasync does not need to update the inode after
    /// each write.
    ///
    /// Default: 0
    pub fn recycle_log_file_num(self, val: usize) -> Self {
        unsafe {
            ll::rocks_dboptions_set_recycle_log_file_num(self.raw, val);
        }
        self
    }

    /// manifest file is rolled over on reaching this limit.
    ///
    /// The older manifest file be deleted.
    ///
    /// The default value is MAX_INT so that roll-over does not take place.
    pub fn max_manifest_file_size(self, val: u64) -> Self {
        unsafe {
            ll::rocks_dboptions_set_max_manifest_file_size(self.raw, val);
        }
        self
    }

    /// Number of shards used for table cache.
    pub fn table_cache_numshardbits(self, val: i32) -> Self {
        unsafe {
            ll::rocks_dboptions_set_table_cache_numshardbits(self.raw, val);
        }
        self
    }

    /// The following two fields affect how archived logs will be deleted.
    ///
    /// 1. If both set to 0, logs will be deleted asap and will not get into
    ///    the archive.
    /// 2. If WAL_ttl_seconds is 0 and WAL_size_limit_MB is not 0,
    ///    WAL files will be checked every 10 min and if total size is greater
    ///    then WAL_size_limit_MB, they will be deleted starting with the
    ///    earliest until size_limit is met. All empty files will be deleted.
    /// 3. If WAL_ttl_seconds is not 0 and WAL_size_limit_MB is 0, then
    ///    WAL files will be checked every WAL_ttl_secondsi / 2 and those that
    ///    are older than WAL_ttl_seconds will be deleted.
    /// 4. If both are not 0, WAL files will be checked every 10 min and both
    ///    checks will be performed with ttl being first.
    pub fn wal_ttl_seconds(self, val: u64) -> Self {
        unsafe {
            ll::rocks_dboptions_set_wal_ttl_seconds(self.raw, val);
        }
        self
    }
    pub fn wal_size_limit_mb(self, val: u64) -> Self {
        unsafe {
            ll::rocks_dboptions_set_wal_size_limit_mb(self.raw, val);
        }
        self
    }

    /// Number of bytes to preallocate (via fallocate) the manifest
    /// files.  Default is 4mb, which is reasonable to reduce random IO
    /// as well as prevent overallocation for mounts that preallocate
    /// large amounts of data (such as xfs's allocsize option).
    pub fn manifest_preallocation_size(self, val: usize) -> Self {
        unsafe {
            ll::rocks_dboptions_set_manifest_preallocation_size(self.raw, val);
        }
        self
    }

    /// Allow the OS to mmap file for reading sst tables. Default: false
    pub fn allow_mmap_reads(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_allow_mmap_reads(self.raw, val as u8);
        }
        self
    }

    /// Allow the OS to mmap file for writing.
    ///
    /// DB::SyncWAL() only works if this is set to false.
    ///
    /// Default: false
    pub fn allow_mmap_writes(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_allow_mmap_writes(self.raw, val as u8);
        }
        self
    }

    /// Enable direct I/O mode for read/write
    /// they may or may not improve performance depending on the use case
    ///
    /// Files will be opened in "direct I/O" mode
    /// which means that data r/w from the disk will not be cached or
    /// bufferized. The hardware buffer of the devices may however still
    /// be used. Memory mapped files are not impacted by these parameters.
    ///
    /// Use O_DIRECT for user reads
    ///
    /// Default: false
    ///
    /// Not supported in ROCKSDB_LITE mode!
    pub fn use_direct_reads(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_use_direct_reads(self.raw, val as u8);
        }
        self
    }

    /// Use O_DIRECT for both reads and writes in background flush and compactions
    /// When true, we also force new_table_reader_for_compaction_inputs to true.
    ///
    /// Default: false
    pub fn use_direct_io_for_flush_and_compaction(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_use_direct_io_for_flush_and_compaction(self.raw, val as u8);
        }
        self
    }

    /// If false, fallocate() calls are bypassed
    pub fn allow_fallocate(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_allow_fallocate(self.raw, val as u8);
        }
        self
    }

    /// Disable child process inherit open files.
    ///
    /// Default: true
    pub fn is_fd_close_on_exec(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_is_fd_close_on_exec(self.raw, val as u8);
        }
        self
    }

    /// if not zero, dump rocksdb.stats to LOG every stats_dump_period_sec
    ///
    /// Default: 600 (10 min)
    pub fn stats_dump_period_sec(self, val: u32) -> Self {
        unsafe {
            ll::rocks_dboptions_set_stats_dump_period_sec(self.raw, val);
        }
        self
    }

    /// If set true, will hint the underlying file system that the file
    /// access pattern is random, when a sst file is opened.
    ///
    /// Default: true
    pub fn advise_random_on_open(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_advise_random_on_open(self.raw, val as u8);
        }
        self
    }

    /// Amount of data to build up in memtables across all column
    /// families before writing to disk.
    ///
    /// This is distinct from write_buffer_size, which enforces a limit
    /// for a single memtable.
    ///
    /// This feature is disabled by default. Specify a non-zero value
    /// to enable it.
    ///
    /// Default: 0 (disabled)
    pub fn db_write_buffer_size(self, val: usize) -> Self {
        unsafe {
            ll::rocks_dboptions_set_db_write_buffer_size(self.raw, val);
        }
        self
    }

    /// The memory usage of memtable will report to this object. The same object
    /// can be passed into multiple DBs and it will track the sum of size of all
    /// the DBs. If the total size of all live memtables of all the DBs exceeds
    /// a limit, a flush will be triggered in the next DB to which the next write
    /// is issued.
    ///
    /// If the object is only passed to on DB, the behavior is the same as
    /// db_write_buffer_size. When write_buffer_manager is set, the value set will
    /// override db_write_buffer_size.
    ///
    /// This feature is disabled by default. Specify a non-zero value
    /// to enable it.
    ///
    /// Default: null
    pub fn write_buffer_manager(self, val: &WriteBufferManager) -> Self {
        unsafe {
            ll::rocks_dboptions_set_write_buffer_manager(self.raw, val.raw());
        }
        self
    }

    /// Specify the file access pattern once a compaction is started.
    /// It will be applied to all input files of a compaction.
    ///
    /// Default: NORMAL
    pub fn access_hint_on_compaction_start(self, val: AccessHint) -> Self {
        unsafe {
            ll::rocks_dboptions_set_access_hint_on_compaction_start(self.raw, mem::transmute(val));
        }
        self
    }

    /// If true, always create a new file descriptor and new table reader
    /// for compaction inputs. Turn this parameter on may introduce extra
    /// memory usage in the table reader, if it allocates extra memory
    /// for indexes. This will allow file descriptor prefetch options
    /// to be set for compaction input files and not to impact file
    /// descriptors for the same file used by user queries.
    ///
    /// Suggest to enable `BlockBasedTableOptions.cache_index_and_filter_blocks`
    /// for this mode if using block-based table.
    ///
    /// Default: false
    pub fn new_table_reader_for_compaction_inputs(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_new_table_reader_for_compaction_inputs(self.raw, val as u8);
        }
        self
    }

    /// If non-zero, we perform bigger reads when doing compaction. If you're
    /// running RocksDB on spinning disks, you should set this to at least 2MB.
    /// That way RocksDB's compaction is doing sequential instead of random reads.
    ///
    /// When non-zero, we also force new_table_reader_for_compaction_inputs to
    /// true.
    ///
    /// Default: 0
    pub fn compaction_readahead_size(self, val: usize) -> Self {
        unsafe {
            ll::rocks_dboptions_set_compaction_readahead_size(self.raw, val);
        }
        self
    }

    /// This is a maximum buffer size that is used by WinMmapReadableFile in
    /// unbuffered disk I/O mode. We need to maintain an aligned buffer for
    /// reads. We allow the buffer to grow until the specified value and then
    /// for bigger requests allocate one shot buffers. In unbuffered mode we
    /// always bypass read-ahead buffer at ReadaheadRandomAccessFile
    /// When read-ahead is required we then make use of compaction_readahead_size
    /// value and always try to read ahead. With read-ahead we always
    /// pre-allocate buffer to the size instead of growing it up to a limit.
    ///
    /// This option is currently honored only on Windows
    ///
    /// Default: 1 Mb
    ///
    /// Special value: 0 - means do not maintain per instance buffer. Allocate
    ///                per request buffer and avoid locking.
    pub fn random_access_max_buffer_size(self, val: usize) -> Self {
        unsafe {
            ll::rocks_dboptions_set_random_access_max_buffer_size(self.raw, val);
        }
        self
    }

    /// This is the maximum buffer size that is used by WritableFileWriter.
    /// On Windows, we need to maintain an aligned buffer for writes.
    /// We allow the buffer to grow until it's size hits the limit in buffered
    /// IO and fix the buffer size when using direct IO to ensure alignment of
    /// write requests if the logical sector size is unusual
    ///
    /// Default: 1024 * 1024 (1 MB)
    pub fn writable_file_max_buffer_size(self, val: usize) -> Self {
        unsafe {
            ll::rocks_dboptions_set_writable_file_max_buffer_size(self.raw, val);
        }
        self
    }

    /// Use adaptive mutex, which spins in the user space before resorting
    /// to kernel. This could reduce context switch when the mutex is not
    /// heavily contended. However, if the mutex is hot, we could end up
    /// wasting spin time.
    ///
    /// Default: false
    pub fn use_adaptive_mutex(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_use_adaptive_mutex(self.raw, val as u8);
        }
        self
    }

    /// Allows OS to incrementally sync files to disk while they are being
    /// written, asynchronously, in the background. This operation can be used
    /// to smooth out write I/Os over time. Users shouldn't rely on it for
    /// persistency guarantee.
    /// Issue one request for every bytes_per_sync written. 0 turns it off.
    /// Default: 0
    ///
    /// You may consider using rate_limiter to regulate write rate to device.
    /// When rate limiter is enabled, it automatically enables bytes_per_sync
    /// to 1MB.
    ///
    /// This option applies to table files
    pub fn bytes_per_sync(self, val: u64) -> Self {
        unsafe {
            ll::rocks_dboptions_set_bytes_per_sync(self.raw, val);
        }
        self
    }

    /// Same as bytes_per_sync, but applies to WAL files
    ///
    /// Default: 0, turned off
    pub fn wal_bytes_per_sync(self, val: u64) -> Self {
        unsafe {
            ll::rocks_dboptions_set_wal_bytes_per_sync(self.raw, val);
        }
        self
    }

    /// A vector of EventListeners which call-back functions will be called
    /// when specific RocksDB event happens.
    pub fn add_listener<T: EventListener>(self, val: T) -> Self {
        unsafe {
            ll::rocks_dboptions_add_listener(
                self.raw,
                Box::into_raw(Box::new(Box::new(val) as Box<EventListener>)) as *mut _,
            );
        }
        self
    }

    /// If true, then the status of the threads involved in this DB will
    /// be tracked and available via GetThreadList() API.
    ///
    /// Default: false
    pub fn enable_thread_tracking(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_enable_thread_tracking(self.raw, val as u8);
        }
        self
    }

    /// The limited write rate to DB if soft_pending_compaction_bytes_limit or
    /// level0_slowdown_writes_trigger is triggered, or we are writing to the
    /// last mem table allowed and we allow more than 3 mem tables. It is
    /// calculated using size of user write requests before compression.
    /// RocksDB may decide to slow down more if the compaction still
    /// gets behind further.
    ///
    /// Unit: byte per second.
    ///
    /// Default: 16MB/s
    pub fn delayed_write_rate(self, val: u64) -> Self {
        unsafe {
            ll::rocks_dboptions_set_delayed_write_rate(self.raw, val);
        }
        self
    }

    /// If true, allow multi-writers to update mem tables in parallel.
    /// Only some memtable_factory-s support concurrent writes; currently it
    /// is implemented only for SkipListFactory.  Concurrent memtable writes
    /// are not compatible with inplace_update_support or filter_deletes.
    /// It is strongly recommended to set enable_write_thread_adaptive_yield
    /// if you are going to use this feature.
    ///
    /// Default: true
    pub fn allow_concurrent_memtable_write(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_allow_concurrent_memtable_write(self.raw, val as u8);
        }
        self
    }

    /// If true, threads synchronizing with the write batch group leader will
    /// wait for up to write_thread_max_yield_usec before blocking on a mutex.
    /// This can substantially improve throughput for concurrent workloads,
    /// regardless of whether allow_concurrent_memtable_write is enabled.
    ///
    /// Default: true
    pub fn enable_write_thread_adaptive_yield(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_enable_write_thread_adaptive_yield(self.raw, val as u8);
        }
        self
    }

    /// The maximum number of microseconds that a write operation will use
    /// a yielding spin loop to coordinate with other write threads before
    /// blocking on a mutex.  (Assuming write_thread_slow_yield_usec is
    /// set properly) increasing this value is likely to increase RocksDB
    /// throughput at the expense of increased CPU usage.
    ///
    /// Default: 100
    pub fn write_thread_max_yield_usec(self, val: u64) -> Self {
        unsafe {
            ll::rocks_dboptions_set_write_thread_max_yield_usec(self.raw, val);
        }
        self
    }

    /// The latency in microseconds after which a std::this_thread::yield
    /// call (sched_yield on Linux) is considered to be a signal that
    /// other processes or threads would like to use the current core.
    /// Increasing this makes writer threads more likely to take CPU
    /// by spinning, which will show up as an increase in the number of
    /// involuntary context switches.
    ///
    /// Default: 3
    pub fn write_thread_slow_yield_usec(self, val: u64) -> Self {
        unsafe {
            ll::rocks_dboptions_set_write_thread_slow_yield_usec(self.raw, val);
        }
        self
    }

    /// If true, then DB::Open() will not update the statistics used to optimize
    /// compaction decision by loading table properties from many files.
    /// Turning off this feature will improve DBOpen time especially in
    /// disk environment.
    ///
    /// Default: false
    pub fn skip_stats_update_on_db_open(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_skip_stats_update_on_db_open(self.raw, val as u8);
        }
        self
    }

    /// Recovery mode to control the consistency while replaying WAL
    ///
    /// Default: PointInTimeRecovery
    pub fn wal_recovery_mode(self, val: WALRecoveryMode) -> Self {
        unsafe {
            ll::rocks_dboptions_set_wal_recovery_mode(self.raw, mem::transmute(val));
        }
        self
    }

    /// if set to false then recovery will fail when a prepared
    /// transaction is encountered in the WAL
    pub fn allow_2pc(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_allow_2pc(self.raw, val as u8);
        }
        self
    }

    /// A global cache for table-level rows.
    ///
    /// Default: nullptr (disabled)
    ///
    /// Not supported in ROCKSDB_LITE mode!
    ///
    /// Rust: will move in and use share_ptr
    pub fn row_cache(self, val: Option<Cache>) -> Self {
        unsafe {
            if let Some(cache) = val {
                ll::rocks_dboptions_set_row_cache(self.raw, cache.raw());
            } else {
                ll::rocks_dboptions_set_row_cache(self.raw, ptr::null_mut());
            }
        }
        self
    }

    // TODO
    // /// A filter object supplied to be invoked while processing write-ahead-logs
    // /// (WALs) during recovery. The filter provides a way to inspect log
    // /// records, ignoring a particular record or skipping replay.
    // /// The filter is invoked at startup and is invoked from a single-thread
    // /// currently.
    // WalFilter* wal_filter ,

    /// If true, then DB::Open / CreateColumnFamily / DropColumnFamily
    /// / SetOptions will fail if options file is not detected or properly
    /// persisted.
    ///
    /// DEFAULT: false
    pub fn fail_if_options_file_error(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_fail_if_options_file_error(self.raw, val as u8);
        }
        self
    }

    /// If true, then print malloc stats together with rocksdb.stats
    /// when printing to LOG.
    ///
    /// DEFAULT: false
    pub fn dump_malloc_stats(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_dump_malloc_stats(self.raw, val as u8);
        }
        self
    }

    /// By default RocksDB replay WAL logs and flush them on DB open, which may
    /// create very small SST files. If this option is enabled, RocksDB will try
    /// to avoid (but not guarantee not to) flush during recovery. Also, existing
    /// WAL logs will be kept, so that if crash happened before flush, we still
    /// have logs to recover from.
    ///
    /// DEFAULT: false
    pub fn avoid_flush_during_recovery(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_avoid_flush_during_recovery(self.raw, val as u8);
        }
        self
    }

    /// By default RocksDB will flush all memtables on DB close if there are
    /// unpersisted data (i.e. with WAL disabled) The flush can be skip to speedup
    /// DB close. Unpersisted data WILL BE LOST.
    ///
    /// DEFAULT: false
    ///
    /// Dynamically changeable through SetDBOptions() API.
    pub fn avoid_flush_during_shutdown(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_avoid_flush_during_shutdown(self.raw, val as u8);
        }
        self
    }

    /// Set this option to true during creation of database if you want
    /// to be able to ingest behind (call IngestExternalFile() skipping keys
    /// that already exist, rather than overwriting matching keys).
    /// Setting this option to true will affect 2 things:
    ///
    /// 1. Disable some internal optimizations around SST file compression
    /// 2. Reserve bottom-most level for ingested files only.
    /// 3. Note that num_levels should be >= 3 if this option is turned on.
    ///
    /// DEFAULT: false
    ///
    /// Immutable.
    pub fn allow_ingest_behind(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_allow_ingest_behind(self.raw, val as u8);
        }
        self
    }

    /// If enabled it uses two queues for writes, one for the ones with
    /// disable_memtable and one for the ones that also write to memtable. This
    /// allows the memtable writes not to lag behind other writes. It can be used
    /// to optimize MySQL 2PC in which only the commits, which are serial, write to
    /// memtable.
    ///
    /// Default: false
    pub fn concurrent_prepare(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_concurrent_prepare(self.raw, val as u8);
        }
        self
    }

    /// If true WAL is not flushed automatically after each write. Instead it
    /// relies on manual invocation of FlushWAL to write the WAL buffer to its
    /// file.
    ///
    /// Default: false
    pub fn manual_wal_flush(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dboptions_set_manual_wal_flush(self.raw, val as u8);
        }
        self
    }
}

/// Options to control the behavior of a database (passed to `DB::Open`)
///
/// ```
/// use rocks::options::Options;
///
/// let _opt = Options::default()
///           .map_db_options(|db| db.create_if_missing(true))
///           .map_cf_options(|cf| cf.disable_auto_compactions(true));
/// ```
pub struct Options {
    raw: *mut ll::rocks_options_t,
}

unsafe impl Sync for Options {}

impl AsRef<Options> for Options {
    fn as_ref(&self) -> &Options {
        self
    }
}

impl Default for Options {
    fn default() -> Self {
        Options { raw: unsafe { ll::rocks_options_create() } }
    }
}

impl Drop for Options {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_options_destroy(self.raw);
        }
    }
}

impl ToRaw<ll::rocks_options_t> for Options {
    fn raw(&self) -> *mut ll::rocks_options_t {
        self.raw
    }
}

impl FromRaw<ll::rocks_options_t> for Options {
    unsafe fn from_ll(raw: *mut ll::rocks_options_t) -> Options {
        Options { raw: raw }
    }
}

impl Options {
    /// default `Options` with `create_if_missing = true`
    #[inline]
    pub fn default_instance() -> &'static Options {
        &*DEFAULT_OPTIONS
    }

    pub fn new(dbopt: Option<DBOptions>, cfopt: Option<ColumnFamilyOptions>) -> Options {
        let dbopt = dbopt.unwrap_or_default();
        let cfopt = cfopt.unwrap_or_default();
        Options { raw: unsafe { ll::rocks_options_create_from_db_cf_options(dbopt.raw(), cfopt.raw()) } }
    }

    // Some functions that make it easier to optimize RocksDB

    /// Configure DBOptions using builder style.
    pub fn map_db_options<F: FnOnce(DBOptions) -> DBOptions>(self, f: F) -> Self {
        let dbopt = unsafe { DBOptions::from_ll(ll::rocks_dboptions_create_from_options(self.raw)) };
        let new_dbopt = f(dbopt);
        let old_cfopt = unsafe { ColumnFamilyOptions::from_ll(ll::rocks_cfoptions_create_from_options(self.raw)) };
        unsafe { Options::from_ll(ll::rocks_options_create_from_db_cf_options(new_dbopt.raw(), old_cfopt.raw())) }
    }

    /// Configure ColumnFamilyOptions using builder style.
    pub fn map_cf_options<F: FnOnce(ColumnFamilyOptions) -> ColumnFamilyOptions>(self, f: F) -> Self {
        let cfopt = unsafe { ColumnFamilyOptions::from_ll(ll::rocks_cfoptions_create_from_options(self.raw)) };
        let new_cfopt = f(cfopt);
        let old_dbopt = unsafe { DBOptions::from_ll(ll::rocks_dboptions_create_from_options(self.raw)) };
        unsafe { Options::from_ll(ll::rocks_options_create_from_db_cf_options(old_dbopt.raw(), new_cfopt.raw())) }
    }

    /// Set appropriate parameters for bulk loading.
    /// The reason that this is a function that returns "this" instead of a
    /// constructor is to enable chaining of multiple similar calls in the future.
    ///

    /// All data will be in level 0 without any automatic compaction.
    /// It's recommended to manually call CompactRange(NULL, NULL) before reading
    /// from the database, because otherwise the read can be very slow.
    pub fn prepare_for_bulk_load(self) -> Self {
        unsafe { ll::rocks_options_prepare_for_bulk_load(self.raw) };
        self
    }

    /// Use this if your DB is very small (like under 1GB) and you don't want to
    /// spend lots of memory for memtables.
    pub fn optimize_for_small_db(self) -> Self {
        unsafe { ll::rocks_options_optimize_for_small_db(self.raw) };
        self
    }
}

/// An application can issue a read request (via Get/Iterators) and specify
/// if that read should process data that ALREADY resides on a specified cache
/// level. For example, if an application specifies kBlockCacheTier then the
/// Get call will process data that is already processed in the memtable or
/// the block cache. It will not page in data from the OS cache or data that
/// resides in storage.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ReadTier {
    /// data in memtable, block cache, OS cache or storage
    ReadAllTier = 0x0,
    /// data in memtable or block cache
    BlockCacheTier = 0x1,
    /// persisted data.  When WAL is disabled, this option
    /// will skip data in memtable.
    /// Note that this ReadTier currently only supports
    /// Get and MultiGet and does not support iterators.
    PersistedTier = 0x2,
}

/// Options that control read operations.
///
/// # Examples
///
/// Construct `ReadOptions` using builder pattern.
///
/// ```no_run
/// use rocks::rocksdb::{ReadOptions, ReadTier};
///
/// let _ropt = ReadOptions::default()
///     .fill_cache(true)
///     .managed(true)
///     .read_tier(ReadTier::PersistedTier);
/// ```
pub struct ReadOptions<'a> {
    raw: *mut ll::rocks_readoptions_t,
    _marker: PhantomData<&'a ()>,
}

unsafe impl<'a> Sync for ReadOptions<'a> {}

impl<'a> AsRef<ReadOptions<'a>> for ReadOptions<'a> {
    fn as_ref(&self) -> &ReadOptions<'a> {
        self
    }
}

impl<'a> Drop for ReadOptions<'a> {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_readoptions_destroy(self.raw);
        }
    }
}

impl<'a> ToRaw<ll::rocks_readoptions_t> for ReadOptions<'a> {
    fn raw(&self) -> *mut ll::rocks_readoptions_t {
        self.raw
    }
}

impl<'a> Default for ReadOptions<'a> {
    fn default() -> Self {
        ReadOptions {
            raw: unsafe { ll::rocks_readoptions_create() },
            _marker: PhantomData,
        }
    }
}

impl<'a> ReadOptions<'a> {
    /// default `ReadOptions` optimization
    #[inline]
    pub fn default_instance() -> &'static ReadOptions<'static> {
        &*DEFAULT_READ_OPTIONS
    }

    pub fn new<'b>(cksum: bool, cache: bool) -> ReadOptions<'b> {
        ReadOptions {
            raw: unsafe { ll::rocks_readoptions_new(cksum as u8, cache as u8) },
            _marker: PhantomData,
        }
    }

    /// If `snapshot` is non-nullptr, read as of the supplied snapshot
    /// (which must belong to the DB that is being read and which must
    /// not have been released).  If `snapshot` is nullptr, use an implicit
    /// snapshot of the state at the beginning of this read operation.
    ///
    /// Default: nullptr
    pub fn snapshot<'s, 'b: 'a, T: AsRef<Snapshot<'s>> + 'b>(self, val: Option<T>) -> Self {
        unsafe {
            ll::rocks_readoptions_set_snapshot(self.raw, val.map(|v| v.as_ref().raw()).unwrap_or(ptr::null_mut()));
        }
        self
    }

    /// `iterate_upper_bound` defines the extent upto which the forward iterator
    /// can returns entries. Once the bound is reached, `is_valid()` will be false.
    /// `iterate_upper_bound` is exclusive ie the bound value is
    /// not a valid entry.  If `iterator_extractor` is not null, the Seek target
    /// and `iterator_upper_bound` need to have the same prefix.
    /// This is because ordering is not guaranteed outside of prefix domain.
    /// There is no lower bound on the iterator. If needed, that can be easily
    /// implemented
    ///
    /// Default: nullptr
    pub fn iterate_upper_bound<'b: 'a>(self, val: &'b [u8]) -> Self {
        unsafe { ll::rocks_readoptions_set_iterate_upper_bound(self.raw, val.as_ptr() as *const _, val.len()) }
        self
    }

    /// If non-zero, NewIterator will create a new table reader which
    /// performs reads of the given size. Using a large size (> 2MB) can
    /// improve the performance of forward iteration on spinning disks.
    ///
    /// Default: 0
    pub fn readahead_size(self, val: usize) -> Self {
        unsafe {
            ll::rocks_readoptions_set_readahead_size(self.raw, val);
        }
        self
    }

    /// A threshold for the number of keys that can be skipped before failing an
    /// iterator seek as incomplete. The default value of 0 should be used to
    /// never fail a request as incomplete, even on skipping too many keys.
    ///
    /// Default: 0
    pub fn max_skippable_internal_keys(self, val: u64) -> Self {
        unsafe {
            ll::rocks_readoptions_set_max_skippable_internal_keys(self.raw, val);
        }
        self
    }

    /// Specify if this read request should process data that ALREADY
    /// resides on a particular cache. If the required data is not
    /// found at the specified cache, then `Status::Incomplete` is returned.
    ///
    /// Default: kReadAllTier
    pub fn read_tier(self, val: ReadTier) -> Self {
        unsafe {
            ll::rocks_readoptions_set_read_tier(self.raw, mem::transmute(val));
        }
        self
    }

    /// If true, all data read from underlying storage will be
    /// verified against corresponding checksums.
    ///
    /// Default: true
    pub fn verify_checksums(self, val: bool) -> Self {
        unsafe {
            ll::rocks_readoptions_set_verify_checksums(self.raw, val as u8);
        }
        self
    }

    /// Should the "data block"/"index block"/"filter block" read for this
    /// iteration be cached in memory?
    ///
    /// Callers may wish to set this field to false for bulk scans.
    ///
    /// Default: true
    pub fn fill_cache(self, val: bool) -> Self {
        unsafe {
            ll::rocks_readoptions_set_fill_cache(self.raw, val as u8);
        }
        self
    }

    /// Specify to create a tailing iterator -- a special iterator that has a
    /// view of the complete database (i.e. it can also be used to read newly
    /// added data) and is optimized for sequential reads. It will return records
    /// that were inserted into the database after the creation of the iterator.
    ///
    /// Default: false
    pub fn tailing(self, val: bool) -> Self {
        unsafe {
            ll::rocks_readoptions_set_tailing(self.raw, val as u8);
        }
        self
    }

    /// Specify to create a managed iterator -- a special iterator that
    /// uses less resources by having the ability to free its underlying
    /// resources on request.
    ///
    /// Default: false
    pub fn managed(self, val: bool) -> Self {
        unsafe {
            ll::rocks_readoptions_set_managed(self.raw, val as u8);
        }
        self
    }

    /// Enable a total order seek regardless of index format (e.g. hash index)
    /// used in the table. Some table format (e.g. plain table) may not support
    /// this option.
    ///
    /// If true when calling `get()`, we also skip prefix bloom when reading from
    /// block based table. It provides a way to read existing data after
    /// changing implementation of prefix extractor.
    pub fn total_order_seek(self, val: bool) -> Self {
        unsafe {
            ll::rocks_readoptions_set_total_order_seek(self.raw, val as u8);
        }
        self
    }

    /// Enforce that the iterator only iterates over the same prefix as the seek.
    /// This option is effective only for prefix seeks, i.e. `prefix_extractor` is
    /// non-null for the column family and `total_order_seek` is false.  Unlike
    /// `iterate_upper_bound`, `prefix_same_as_start` only works within a prefix
    /// but in both directions.
    ///
    /// Default: false
    pub fn prefix_same_as_start(self, val: bool) -> Self {
        unsafe {
            ll::rocks_readoptions_set_prefix_same_as_start(self.raw, val as u8);
        }
        self
    }

    /// Keep the blocks loaded by the iterator pinned in memory as long as the
    /// iterator is not deleted, If used when reading from tables created with
    /// `BlockBasedTableOptions::use_delta_encoding = false`,
    /// Iterator's property `"rocksdb.iterator.is-key-pinned"` is guaranteed to
    /// return 1.
    ///
    /// Default: false
    pub fn pin_data(self, val: bool) -> Self {
        unsafe {
            ll::rocks_readoptions_set_pin_data(self.raw, val as u8);
        }
        self
    }

    /// If true, when `PurgeObsoleteFile` is called in `CleanupIteratorState`, we
    /// schedule a background job in the flush job queue and delete obsolete files
    /// in background.
    ///
    /// Default: false
    pub fn background_purge_on_iterator_cleanup(self, val: bool) -> Self {
        unsafe {
            ll::rocks_readoptions_set_background_purge_on_iterator_cleanup(self.raw, val as u8);
        }
        self
    }


    /// If true, keys deleted using the `delete_range()` API will be visible to
    /// readers until they are naturally deleted during compaction. This improves
    /// read performance in DBs with many range deletions.
    ///
    /// Default: false
    pub fn ignore_range_deletions(self, val: bool) -> Self {
        unsafe {
            ll::rocks_readoptions_set_ignore_range_deletions(self.raw, val as u8);
        }
        self
    }
}

/// Options that control write operations
pub struct WriteOptions {
    raw: *mut ll::rocks_writeoptions_t,
}

unsafe impl Sync for WriteOptions {}

impl AsRef<WriteOptions> for WriteOptions {
    fn as_ref(&self) -> &WriteOptions {
        self
    }
}

impl Default for WriteOptions {
    fn default() -> Self {
        WriteOptions { raw: unsafe { ll::rocks_writeoptions_create() } }
    }
}

impl Drop for WriteOptions {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_writeoptions_destroy(self.raw);
        }
    }
}

impl ToRaw<ll::rocks_writeoptions_t> for WriteOptions {
    fn raw(&self) -> *mut ll::rocks_writeoptions_t {
        self.raw
    }
}

impl WriteOptions {
    /// default `WriteOptions` optimization
    #[inline]
    pub fn default_instance() -> &'static WriteOptions {
        &*DEFAULT_WRITE_OPTIONS
    }

    /// If true, the write will be flushed from the operating system
    /// buffer cache (by calling `WritableFile::Sync()`) before the write
    /// is considered complete.  If this flag is true, writes will be
    /// slower.
    ///
    /// If this flag is false, and the machine crashes, some recent
    /// writes may be lost.  Note that if it is just the process that
    /// crashes (i.e., the machine does not reboot), no writes will be
    /// lost even if sync==false.
    ///
    /// In other words, a DB write with sync==false has similar
    /// crash semantics as the "`write()`" system call.  A DB write
    /// with `sync==true` has similar crash semantics to a "`write()`"
    /// system call followed by "`fdatasync()`".
    ///
    /// Default: false
    pub fn sync(self, val: bool) -> Self {
        unsafe {
            ll::rocks_writeoptions_set_sync(self.raw, val as u8);
        }
        self
    }

    /// If true, writes will not first go to the write ahead log,
    /// and the write may got lost after a crash.
    pub fn disable_wal(self, val: bool) -> Self {
        unsafe {
            ll::rocks_writeoptions_set_disable_wal(self.raw, val as u8);
        }
        self
    }

    /// If true and if user is trying to write to column families that don't exist
    /// (they were dropped),  ignore the write (don't return an error). If there
    /// are multiple writes in a WriteBatch, other writes will succeed.
    ///
    /// Default: false
    pub fn ignore_missing_column_families(self, val: bool) -> Self {
        unsafe {
            ll::rocks_writeoptions_set_ignore_missing_column_families(self.raw, val as u8);
        }
        self
    }

    /// If true and we need to wait or sleep for the write request, fails
    /// immediately with Status::Incomplete().
    pub fn no_slowdown(self, val: bool) -> Self {
        unsafe {
            ll::rocks_writeoptions_set_no_slowdown(self.raw, val as u8);
        }
        self
    }

    /// If true, this write request is of lower priority if compaction is
    /// behind. In this case, no_slowdown = true, the request will be cancelled
    /// immediately with Status::Incomplete() returned. Otherwise, it will be
    /// slowed down. The slowdown value is determined by RocksDB to guarantee
    /// it introduces minimum impacts to high priority writes.
    ///
    /// Default: false
    pub fn low_pri(self, val: bool) -> Self {
        unsafe {
            ll::rocks_writeoptions_set_low_pri(self.raw, val as u8);
        }
        self
    }
}

/// Options that control flush operations
#[repr(C)]
pub struct FlushOptions {
    raw: *mut ll::rocks_flushoptions_t,
}

impl Default for FlushOptions {
    fn default() -> Self {
        FlushOptions { raw: unsafe { ll::rocks_flushoptions_create() } }
    }
}

impl Drop for FlushOptions {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_flushoptions_destroy(self.raw);
        }
    }
}

impl ToRaw<ll::rocks_flushoptions_t> for FlushOptions {
    fn raw(&self) -> *mut ll::rocks_flushoptions_t {
        self.raw
    }
}

impl FlushOptions {
    /// If true, the flush will wait until the flush is done.
    /// Default: true
    pub fn wait(self, val: bool) -> Self {
        unsafe {
            ll::rocks_flushoptions_set_wait(self.raw, val as u8);
        }
        self
    }
}

unsafe impl Sync for FlushOptions {}

/// `CompactionOptions` are used in `CompactFiles()` call.
#[repr(C)]
pub struct CompactionOptions {
    raw: *mut ll::rocks_compaction_options_t,
}

impl ToRaw<ll::rocks_compaction_options_t> for CompactionOptions {
    fn raw(&self) -> *mut ll::rocks_compaction_options_t {
        self.raw
    }
}

impl Default for CompactionOptions {
    fn default() -> Self {
        CompactionOptions::new()
    }
}

impl Drop for CompactionOptions {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_compaction_options_destroy(self.raw);
        }
    }
}

impl CompactionOptions {
    pub fn new() -> CompactionOptions {
        CompactionOptions { raw: unsafe { ll::rocks_compaction_options_create() } }
    }

    /// Compaction output compression type
    ///
    /// Default: snappy
    pub fn compression(self, val: CompressionType) -> Self {
        unsafe {
            ll::rocks_compaction_options_set_compression(self.raw, mem::transmute(val));
        }
        self
    }

    /// Compaction will create files of size `output_file_size_limit`.
    ///
    /// Default: MAX, which means that compaction will create a single file
    pub fn output_file_size_limit(self, val: u64) -> Self {
        unsafe {
            ll::rocks_compaction_options_set_output_file_size_limit(self.raw, val);
        }
        self
    }
}

unsafe impl Sync for CompactionOptions {}

/// For level based compaction, we can configure if we want to skip/force
/// bottommost level compaction.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BottommostLevelCompaction {
    /// Skip bottommost level compaction
    Skip,
    /// Only compact bottommost level if there is a compaction filter
    /// This is the default option
    IfHaveCompactionFilter,
    /// Always compact bottommost level
    Force,
}

/// `CompactRangeOptions` is used by `compact_range()` call.
pub struct CompactRangeOptions {
    raw: *mut ll::rocks_compactrange_options_t,
}

impl Default for CompactRangeOptions {
    fn default() -> Self {
        CompactRangeOptions { raw: unsafe { ll::rocks_compactrange_options_create() } }
    }
}

impl Drop for CompactRangeOptions {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_compactrange_options_destroy(self.raw);
        }
    }
}

impl ToRaw<ll::rocks_compactrange_options_t> for CompactRangeOptions {
    fn raw(&self) -> *mut ll::rocks_compactrange_options_t {
        self.raw
    }
}

impl CompactRangeOptions {
    /// If true, no other compaction will run at the same time as this
    /// manual compaction
    pub fn exclusive_manual_compaction(self, val: bool) -> Self {
        unsafe {
            ll::rocks_compactrange_options_set_exclusive_manual_compaction(self.raw, val as u8);
        }
        self
    }

    /// If true, compacted files will be moved to the minimum level capable
    /// of holding the data or given level (specified non-negative target_level).
    pub fn change_level(self, val: bool) -> Self {
        unsafe {
            ll::rocks_compactrange_options_set_change_level(self.raw, val as u8);
        }
        self
    }

    /// If change_level is true and target_level have non-negative value, compacted
    /// files will be moved to target_level.
    pub fn target_level(self, val: i32) -> Self {
        unsafe {
            ll::rocks_compactrange_options_set_target_level(self.raw, val);
        }
        self
    }

    /// Compaction outputs will be placed in options.db_paths[target_path_id].
    /// Behavior is undefined if target_path_id is out of range.
    pub fn target_path_id(self, val: u32) -> Self {
        unsafe {
            ll::rocks_compactrange_options_set_target_path_id(self.raw, val);
        }
        self
    }

    /// By default level based compaction will only compact the bottommost level
    /// if there is a compaction filter
    pub fn bottommost_level_compaction(self, val: BottommostLevelCompaction) -> Self {
        unsafe {
            ll::rocks_compactrange_options_set_bottommost_level_compaction(self.raw, mem::transmute(val));
        }
        self
    }
}

unsafe impl Sync for CompactRangeOptions {}

/// `IngestExternalFileOptions` is used by `ingest_external_file()`
#[repr(C)]
pub struct IngestExternalFileOptions {
    raw: *mut ll::rocks_ingestexternalfile_options_t,
}

impl Default for IngestExternalFileOptions {
    fn default() -> Self {
        IngestExternalFileOptions { raw: unsafe { ll::rocks_ingestexternalfile_options_create() } }
    }
}

impl Drop for IngestExternalFileOptions {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_ingestexternalfile_options_destroy(self.raw);
        }
    }
}

impl ToRaw<ll::rocks_ingestexternalfile_options_t> for IngestExternalFileOptions {
    fn raw(&self) -> *mut ll::rocks_ingestexternalfile_options_t {
        self.raw
    }
}

impl IngestExternalFileOptions {
    /// Can be set to true to move the files instead of copying them.
    pub fn move_files(self, val: bool) -> Self {
        unsafe {
            ll::rocks_ingestexternalfile_options_set_move_files(self.raw, val as u8);
        }
        self
    }

    /// If set to false, an ingested file keys could appear in existing snapshots
    /// that where created before the file was ingested.
    pub fn snapshot_consistency(self, val: bool) -> Self {
        unsafe {
            ll::rocks_ingestexternalfile_options_set_snapshot_consistency(self.raw, val as u8);
        }
        self
    }

    /// If set to false, IngestExternalFile() will fail if the file key range
    /// overlaps with existing keys or tombstones in the DB.
    pub fn allow_global_seqno(self, val: bool) -> Self {
        unsafe {
            ll::rocks_ingestexternalfile_options_set_allow_global_seqno(self.raw, val as u8);
        }
        self
    }

    /// If set to false and the file key range overlaps with the memtable key range
    /// (memtable flush required), IngestExternalFile will fail.
    pub fn allow_blocking_flush(self, val: bool) -> Self {
        unsafe {
            ll::rocks_ingestexternalfile_options_set_allow_blocking_flush(self.raw, val as u8);
        }
        self
    }

    /// Set to true if you would like duplicate keys in the file being ingested
    /// to be skipped rather than overwriting existing data under that key.
    /// Usecase: back-fill of some historical data in the database without
    /// over-writing existing newer version of data.
    ///
    /// This option could only be used if the DB has been running
    /// with allow_ingest_behind=true since the dawn of time.
    /// All files will be ingested at the bottommost level with seqno=0.
    pub fn ingest_behind(self, val: bool) -> Self {
        unsafe {
            ll::rocks_ingestexternalfile_options_set_ingest_behind(self.raw, val as u8);
        }
        self
    }
}

unsafe impl Sync for IngestExternalFileOptions {}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::rocksdb::*;

    #[test]
    fn dboptions_stringify() {
        let opts = DBOptions::default().allow_2pc(true);
        assert!(format!("{}", opts).contains("allow_2pc=true"));
    }

    #[test]
    fn cfoptions_stringify() {
        let opts = ColumnFamilyOptions::default().max_write_buffer_number(5);
        assert!(format!("{}", opts).contains("max_write_buffer_number=5"));
    }

    #[test]
    fn readoptions() {
        // FIXME: is disable block cache works?
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(
            Options::default()
                .map_db_options(|db| db.create_if_missing(true))
                .map_cf_options(|cf| {
                    cf.table_factory_block_based(
                        BlockBasedTableOptions::default()
                            .no_block_cache(true)
                            .block_cache(None),
                    )
                }),
            &tmp_dir,
        ).unwrap();
        assert!(
            db.put(&Default::default(), b"long-key", vec![b'A'; 1024].as_ref())
                .is_ok()
        );
        assert!(db.compact_range(&Default::default(), ..).is_ok());
        let val = db.get(&ReadOptions::default().read_tier(ReadTier::BlockCacheTier), b"long-key");
        assert!(val.is_ok());
    }

    #[test]
    fn default_instance() {
        let w1 = WriteOptions::default_instance();
        let w2 = WriteOptions::default_instance();

        assert_eq!(w1.raw, w2.raw);

        let w1 = ReadOptions::default_instance();
        let w2 = ReadOptions::default_instance();

        assert_eq!(w1.raw, w2.raw);
    }


    #[test]
    fn compact_range_options() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(Options::default().map_db_options(|db| db.create_if_missing(true)), &tmp_dir).unwrap();
        assert!(
            db.put(&Default::default(), b"long-key", vec![b'A'; 1024 * 1024].as_ref())
                .is_ok()
        );
        assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
        assert!(
            db.put(&Default::default(), b"long-key-2", vec![b'A'; 2 * 1024].as_ref())
                .is_ok()
        );

        assert!(
            db.compact_range(
                &CompactRangeOptions::default()
                    .change_level(true)
                    .target_level(4), // TO level 4
                ..,
            ).is_ok()
        );

        let meta = db.get_column_family_metadata(&db.default_column_family());
        println!("Meta => {:?}", meta);
        assert_eq!(meta.levels.len(), 7, "default level num");
        assert_eq!(meta.levels[0].files.len(), 0);
        assert_eq!(meta.levels[1].files.len(), 0);
        assert_eq!(meta.levels[2].files.len(), 0);
        assert_eq!(meta.levels[3].files.len(), 0);
        assert!(meta.levels[4].files.len() > 0);
    }
}
