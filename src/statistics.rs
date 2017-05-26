//! Analyze the performance of a DB

use std::fmt;
use std::mem;
use std::os::raw::c_void;

use rocks_sys as ll;

use to_raw::ToRaw;


#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Tickers {
    /// total block cache misses
    ///
    /// ```text
    /// REQUIRES: BLOCK_CACHE_MISS == BLOCK_CACHE_INDEX_MISS +
    ///                               BLOCK_CACHE_FILTER_MISS +
    ///                               BLOCK_CACHE_DATA_MISS;
    /// ```
    BlockCacheMiss = 0,
    /// total block cache hit
    ///
    /// ```text
    /// REQUIRES: BLOCK_CACHE_HIT == BLOCK_CACHE_INDEX_HIT +
    ///                              BLOCK_CACHE_FILTER_HIT +
    ///                              BLOCK_CACHE_DATA_HIT;
    /// ```
    BlockCacheHit,
    /// # of blocks added to block cache.
    BlockCacheAdd,
    /// # of failures when adding blocks to block cache.
    BlockCacheAddFailures,
    /// # of times cache miss when accessing index block from block cache.
    BlockCacheIndexMiss,
    /// # of times cache hit when accessing index block from block cache.
    BlockCacheIndexHit,
    /// # of index blocks added to block cache.
    BlockCacheIndexAdd,
    /// # of bytes of index blocks inserted into cache
    BlockCacheIndexBytesInsert,
    /// # of bytes of index block erased from cache
    BlockCacheIndexBytesEvict,
    /// # of times cache miss when accessing filter block from block cache.
    BlockCacheFilterMiss,
    /// # of times cache hit when accessing filter block from block cache.
    BlockCacheFilterHit,
    /// # of filter blocks added to block cache.
    BlockCacheFilterAdd,
    /// # of bytes of bloom filter blocks inserted into cache
    BlockCacheFilterBytesInsert,
    /// # of bytes of bloom filter block erased from cache
    BlockCacheFilterBytesEvict,
    /// # of times cache miss when accessing data block from block cache.
    BlockCacheDataMiss,
    /// # of times cache hit when accessing data block from block cache.
    BlockCacheDataHit,
    /// # of data blocks added to block cache.
    BlockCacheDataAdd,
    /// # of bytes of data blocks inserted into cache
    BlockCacheDataBytesInsert,
    /// # of bytes read from cache.
    BlockCacheBytesRead,
    /// # of bytes written into cache.
    BlockCacheBytesWrite,

    /// # of times bloom filter has avoided file reads.
    BloomFilterUseful,

    /// # persistent cache hit
    PersistentCacheHit,
    /// # persistent cache miss
    PersistentCacheMiss,

    /// # total simulation block cache hits
    SimBlockCacheHit,
    /// # total simulation block cache misses
    SimBlockCacheMiss,

    /// # of memtable hits.
    MemtableHit,
    /// # of memtable misses.
    MemtableMiss,

    /// # of Get() queries served by L0
    GetHitL0,
    /// # of Get() queries served by L1
    GetHitL1,
    /// # of Get() queries served by L2 and up
    GetHitL2AndUp,

    /// COMPACTION_KEY_DROP_* count the reasons for key drop during compaction
    ///
    /// There are 4 reasons currently.
    ///
    /// key was written with a newer value.
    /// Also includes keys dropped for range del.
    CompactionKeyDropNewerEntry,
    /// The key is obsolete.
    CompactionKeyDropObsolete,
    /// key was covered by a range tombstone.
    CompactionKeyDropRangeDel,
    /// user compaction function has dropped the key.
    CompactionKeyDropUser,
    /// all keys in range were deleted.
    CompactionRangeDelDropObsolete,

    /// Number of keys written to the database via the Put and Write call's
    NumberKeysWritten,
    /// Number of Keys read,
    NumberKeysRead,
    /// Number keys updated, if inplace update is enabled
    NumberKeysUpdated,
    /// the number of uncompressed bytes issued by `DB::Put()`, `DB::Delete()`,
    /// `DB::Merge()`, and `DB::Write()`.
    BytesWritten,
    /// The number of uncompressed bytes read from `DB::Get()`.  It could be
    /// either from memtables, cache, or table files.
    /// 
    /// For the number of logical bytes read from `DB::MultiGet()`,
    /// please use `NUMBER_MULTIGET_BYTES_READ`.
    BytesRead,
    /// The number of calls to seek/next/prev
    NumberDbSeek,
    NumberDbNext,
    NumberDbPrev,
    /// The number of calls to seek/next/prev that returned data
    NumberDbSeekFound,
    NumberDbNextFound,
    NumberDbPrevFound,
    /// The number of uncompressed bytes read from an iterator.
    /// Includes size of key and value.
    IterBytesRead,
    NoFileCloses,
    NoFileOpens,
    NoFileErrors,
    /// DEPRECATED Time system had to wait to do LO-L1 compactions
    StallL0SlowdownMicros,
    /// DEPRECATED Time system had to wait to move memtable to L1.
    StallMemtableCompactionMicros,
    /// DEPRECATED write throttle because of too many files in L0
    StallL0NumFilesMicros,
    /// Writer has to wait for compaction or flush to finish.
    StallMicros,
    /// The wait time for db mutex.
    /// Disabled by default. To enable it set stats level to kAll
    DbMutexWaitMicros,
    RateLimitDelayMillis,
    /// number of iterators currently open
    NoIterators,

    /// Number of MultiGet calls, keys read, and bytes read
    NumberMultigetCalls,
    NumberMultigetKeysRead,
    NumberMultigetBytesRead,

    /// Number of deletes records that were not required to be
    /// written to storage because key does not exist
    NumberFilteredDeletes,
    NumberMergeFailures,

    /// number of times bloom was checked before creating iterator on a
    /// file, and the number of times the check was useful in avoiding
    /// iterator creation (and thus likely IOPs).
    BloomFilterPrefixChecked,
    BloomFilterPrefixUseful,

    /// Number of times we had to reseek inside an iteration to skip
    /// over large number of keys with same userkey.
    NumberOfReseeksInIteration,

    /// Record the number of calls to GetUpadtesSince. Useful to keep track of
    /// transaction log iterator refreshes
    GetUpdatesSinceCalls,
    /// miss in the compressed block cache
    BlockCacheCompressedMiss,
    /// hit in the compressed block cache
    BlockCacheCompressedHit,
    /// Number of blocks added to comopressed block cache
    BlockCacheCompressedAdd,
    /// Number of failures when adding blocks to compressed block cache
    BlockCacheCompressedAddFailures,
    /// Number of times WAL sync is done
    WalFileSynced,
    /// Number of bytes written to WAL
    WalFileBytes,

    /// Writes can be processed by requesting thread or by the thread at the
    /// head of the writers queue.
    WriteDoneBySelf,
    /// Equivalent to writes done for others
    WriteDoneByOther,
    /// Number of writes ending up with timed-out.
    WriteTimedout,
    /// Number of Write calls that request WAL
    WriteWithWal,
    /// Bytes read during compaction
    CompactReadBytes,
    /// Bytes written during compaction
    CompactWriteBytes,
    /// Bytes written during flush
    FlushWriteBytes,

    /// Number of table's properties loaded directly from file, without creating
    /// table reader object.
    NumberDirectLoadTableProperties,
    NumberSuperversionAcquires,
    NumberSuperversionReleases,
    NumberSuperversionCleanups,

    /// # of compressions/decompressions executed
    NumberBlockCompressed,
    NumberBlockDecompressed,

    NumberBlockNotCompressed,
    MergeOperationTotalTime,
    FilterOperationTotalTime,

    /// Row cache.
    RowCacheHit,
    RowCacheMiss,

    /// Read amplification statistics.
    ///
    /// Read amplification can be calculated using this formula
    /// `(READ_AMP_TOTAL_READ_BYTES / READ_AMP_ESTIMATE_USEFUL_BYTES)`
    ///
    /// REQUIRES: `ReadOptions::read_amp_bytes_per_bit` to be enabled
    ///
    /// Estimate of total bytes actually used.
    ReadAmpEstimateUsefulBytes,
    /// Total size of loaded data blocks.
    ReadAmpTotalReadBytes,

    /// Number of refill intervals where rate limiter's bytes are fully consumed.
    NumberRateLimiterDrains,
}

impl fmt::Display for Tickers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Tickers::*;

        let val = match *self {
            BlockCacheMiss                  => "rocksdb.block.cache.miss",
            BlockCacheHit                   => "rocksdb.block.cache.hit",
            BlockCacheAdd                   => "rocksdb.block.cache.add",
            BlockCacheAddFailures           => "rocksdb.block.cache.add.failures",
            BlockCacheIndexMiss             => "rocksdb.block.cache.index.miss",
            BlockCacheIndexHit              => "rocksdb.block.cache.index.hit",
            BlockCacheIndexAdd              => "rocksdb.block.cache.index.add",
            BlockCacheIndexBytesInsert      => "rocksdb.block.cache.index.bytes.insert",
            BlockCacheIndexBytesEvict       => "rocksdb.block.cache.index.bytes.evict",
            BlockCacheFilterMiss            => "rocksdb.block.cache.filter.miss",
            BlockCacheFilterHit             => "rocksdb.block.cache.filter.hit",
            BlockCacheFilterAdd             => "rocksdb.block.cache.filter.add",
            BlockCacheFilterBytesInsert     => "rocksdb.block.cache.filter.bytes.insert",
            BlockCacheFilterBytesEvict      => "rocksdb.block.cache.filter.bytes.evict",
            BlockCacheDataMiss              => "rocksdb.block.cache.data.miss",
            BlockCacheDataHit               => "rocksdb.block.cache.data.hit",
            BlockCacheDataAdd               => "rocksdb.block.cache.data.add",
            BlockCacheDataBytesInsert       => "rocksdb.block.cache.data.bytes.insert",
            BlockCacheBytesRead             => "rocksdb.block.cache.bytes.read",
            BlockCacheBytesWrite            => "rocksdb.block.cache.bytes.write",
            BloomFilterUseful               => "rocksdb.bloom.filter.useful",
            PersistentCacheHit              => "rocksdb.persistent.cache.hit",
            PersistentCacheMiss             => "rocksdb.persistent.cache.miss",
            SimBlockCacheHit                => "rocksdb.sim.block.cache.hit",
            SimBlockCacheMiss               => "rocksdb.sim.block.cache.miss",
            MemtableHit                     => "rocksdb.memtable.hit",
            MemtableMiss                    => "rocksdb.memtable.miss",
            GetHitL0                        => "rocksdb.l0.hit",
            GetHitL1                        => "rocksdb.l1.hit",
            GetHitL2AndUp                   => "rocksdb.l2andup.hit",
            CompactionKeyDropNewerEntry     => "rocksdb.compaction.key.drop.new",
            CompactionKeyDropObsolete       => "rocksdb.compaction.key.drop.obsolete",
            CompactionKeyDropRangeDel       => "rocksdb.compaction.key.drop.range_del",
            CompactionKeyDropUser           => "rocksdb.compaction.key.drop.user",
            CompactionRangeDelDropObsolete  => "rocksdb.compaction.range_del.drop.obsolete",
            NumberKeysWritten               => "rocksdb.number.keys.written",
            NumberKeysRead                  => "rocksdb.number.keys.read",
            NumberKeysUpdated               => "rocksdb.number.keys.updated",
            BytesWritten                    => "rocksdb.bytes.written",
            BytesRead                       => "rocksdb.bytes.read",
            NumberDbSeek                    => "rocksdb.number.db.seek",
            NumberDbNext                    => "rocksdb.number.db.next",
            NumberDbPrev                    => "rocksdb.number.db.prev",
            NumberDbSeekFound               => "rocksdb.number.db.seek.found",
            NumberDbNextFound               => "rocksdb.number.db.next.found",
            NumberDbPrevFound               => "rocksdb.number.db.prev.found",
            IterBytesRead                   => "rocksdb.db.iter.bytes.read",
            NoFileCloses                    => "rocksdb.no.file.closes",
            NoFileOpens                     => "rocksdb.no.file.opens",
            NoFileErrors                    => "rocksdb.no.file.errors",
            StallL0SlowdownMicros           => "rocksdb.l0.slowdown.micros",
            StallMemtableCompactionMicros   => "rocksdb.memtable.compaction.micros",
            StallL0NumFilesMicros           => "rocksdb.l0.num.files.stall.micros",
            StallMicros                     => "rocksdb.stall.micros",
            DbMutexWaitMicros               => "rocksdb.db.mutex.wait.micros",
            RateLimitDelayMillis            => "rocksdb.rate.limit.delay.millis",
            NoIterators                     => "rocksdb.num.iterators",
            NumberMultigetCalls             => "rocksdb.number.multiget.get",
            NumberMultigetKeysRead          => "rocksdb.number.multiget.keys.read",
            NumberMultigetBytesRead         => "rocksdb.number.multiget.bytes.read",
            NumberFilteredDeletes           => "rocksdb.number.deletes.filtered",
            NumberMergeFailures             => "rocksdb.number.merge.failures",
            BloomFilterPrefixChecked        => "rocksdb.bloom.filter.prefix.checked",
            BloomFilterPrefixUseful         => "rocksdb.bloom.filter.prefix.useful",
            NumberOfReseeksInIteration      => "rocksdb.number.reseeks.iteration",
            GetUpdatesSinceCalls            => "rocksdb.getupdatessince.calls",
            BlockCacheCompressedMiss        => "rocksdb.block.cachecompressed.miss",
            BlockCacheCompressedHit         => "rocksdb.block.cachecompressed.hit",
            BlockCacheCompressedAdd         => "rocksdb.block.cachecompressed.add",
            BlockCacheCompressedAddFailures => "rocksdb.block.cachecompressed.add.failures",
            WalFileSynced                   => "rocksdb.wal.synced",
            WalFileBytes                    => "rocksdb.wal.bytes",
            WriteDoneBySelf                 => "rocksdb.write.self",
            WriteDoneByOther                => "rocksdb.write.other",
            WriteTimedout                   => "rocksdb.write.timeout",
            WriteWithWal                    => "rocksdb.write.wal",
            CompactReadBytes                => "rocksdb.compact.read.bytes",
            CompactWriteBytes               => "rocksdb.compact.write.bytes",
            FlushWriteBytes                 => "rocksdb.flush.write.bytes",
            NumberDirectLoadTableProperties => "rocksdb.number.direct.load.table.properties",
            NumberSuperversionAcquires      => "rocksdb.number.superversion_acquires",
            NumberSuperversionReleases      => "rocksdb.number.superversion_releases",
            NumberSuperversionCleanups      => "rocksdb.number.superversion_cleanups",
            NumberBlockCompressed           => "rocksdb.number.block.compressed",
            NumberBlockDecompressed         => "rocksdb.number.block.decompressed",
            NumberBlockNotCompressed        => "rocksdb.number.block.not_compressed",
            MergeOperationTotalTime         => "rocksdb.merge.operation.time.nanos",
            FilterOperationTotalTime        => "rocksdb.filter.operation.time.nanos",
            RowCacheHit                     => "rocksdb.row.cache.hit",
            RowCacheMiss                    => "rocksdb.row.cache.miss",
            ReadAmpEstimateUsefulBytes      => "rocksdb.read.amp.estimate.useful.bytes",
            ReadAmpTotalReadBytes           => "rocksdb.read.amp.total.read.bytes",
            NumberRateLimiterDrains         => "rocksdb.number.rate_limiter.drains",
        };
        write!(f, "{}", val)
    }
}



#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Histograms {
    DbGet = 0,
    DbWrite,
    CompactionTime,
    SubcompactionSetupTime,
    TableSyncMicros,
    CompactionOutfileSyncMicros,
    WalFileSyncMicros,
    ManifestFileSyncMicros,
    /// TIME SPENT IN IO DURING TABLE OPEN
    TableOpenIoMicros,
    DbMultiget,
    ReadBlockCompactionMicros,
    ReadBlockGetMicros,
    WriteRawBlockMicros,
    StallL0SlowdownCount,
    StallMemtableCompactionCount,
    StallL0NumFilesCount,
    HardRateLimitDelayCount,
    SoftRateLimitDelayCount,
    NumFilesInSingleCompaction,
    DbSeek,
    WriteStall,
    SstReadMicros,
    /// The number of subcompactions actually scheduled during a compaction
    NumSubcompactionsScheduled,
    /// Value size distribution in each operation
    BytesPerRead,
    BytesPerWrite,
    BytesPerMultiget,

    /// number of bytes compressed/decompressed
    /// number of bytes is when uncompressed; i.e. before/after respectively
    BytesCompressed,
    BytesDecompressed,
    CompressionTimesNanos,
    DecompressionTimesNanos,
}


impl fmt::Display for Histograms {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Histograms::*;

        let val = match *self {
            DbGet                        => "rocksdb.db.get.micros",
            DbWrite                      => "rocksdb.db.write.micros",
            CompactionTime               => "rocksdb.compaction.times.micros",
            SubcompactionSetupTime       => "rocksdb.subcompaction.setup.times.micros",
            TableSyncMicros              => "rocksdb.table.sync.micros",
            CompactionOutfileSyncMicros  => "rocksdb.compaction.outfile.sync.micros",
            WalFileSyncMicros            => "rocksdb.wal.file.sync.micros",
            ManifestFileSyncMicros       => "rocksdb.manifest.file.sync.micros",
            TableOpenIoMicros            => "rocksdb.table.open.io.micros",
            DbMultiget                   => "rocksdb.db.multiget.micros",
            ReadBlockCompactionMicros    => "rocksdb.read.block.compaction.micros",
            ReadBlockGetMicros           => "rocksdb.read.block.get.micros",
            WriteRawBlockMicros          => "rocksdb.write.raw.block.micros",
            StallL0SlowdownCount         => "rocksdb.l0.slowdown.count",
            StallMemtableCompactionCount => "rocksdb.memtable.compaction.count",
            StallL0NumFilesCount         => "rocksdb.num.files.stall.count",
            HardRateLimitDelayCount      => "rocksdb.hard.rate.limit.delay.count",
            SoftRateLimitDelayCount      => "rocksdb.soft.rate.limit.delay.count",
            NumFilesInSingleCompaction   => "rocksdb.numfiles.in.singlecompaction",
            DbSeek                       => "rocksdb.db.seek.micros",
            WriteStall                   => "rocksdb.db.write.stall",
            SstReadMicros                => "rocksdb.sst.read.micros",
            NumSubcompactionsScheduled   => "rocksdb.num.subcompactions.scheduled",
            BytesPerRead                 => "rocksdb.bytes.per.read",
            BytesPerWrite                => "rocksdb.bytes.per.write",
            BytesPerMultiget             => "rocksdb.bytes.per.multiget",
            BytesCompressed              => "rocksdb.bytes.compressed",
            BytesDecompressed            => "rocksdb.bytes.decompressed",
            CompressionTimesNanos        => "rocksdb.compression.times.nanos",
            DecompressionTimesNanos      => "rocksdb.decompression.times.nanos",
        };
        write!(f, "{}", val)
    }
}


#[repr(C)]
pub struct HistogramData {
    pub median: f64,
    pub percentile95: f64,
    pub percentile99: f64,
    pub average: f64,
    pub standard_deviation: f64,
    pub max: f64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum StatsLevel {
    /// Collect all stats except time inside mutex lock AND time spent on
    /// compression.
    ExceptDetailedTimers,
    /// Collect all stats except the counters requiring to get time inside the
    /// mutex lock.
    ExceptTimeForMutex,
    /// Collect all stats, including measuring duration of mutex operations.
    /// If getting time is expensive on the platform to run, it can
    /// reduce scalability to more threads, especially for writes.
    All,
}

/// Analyze the performance of a db
pub struct Statistics {
    raw: *mut ll::rocks_statistics_t,
}

impl Drop for Statistics {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_statistics_destroy(self.raw);
        }
    }
}

// Clone for shared access?
impl Clone for Statistics {
    fn clone(&self) -> Self {
        Statistics {
            raw: unsafe { ll::rocks_statistics_copy(self.raw) },
        }
    }
}

impl ToRaw<ll::rocks_statistics_t> for Statistics {
    fn raw(&self) -> *mut ll::rocks_statistics_t {
        self.raw
    }
}

impl Statistics {
    pub fn new() -> Statistics {
        Statistics {
            raw: unsafe { ll::rocks_statistics_create() },
        }
    }

    pub fn get_ticker_count(&self, ticker_type: Tickers) -> u64 {
        unsafe {
            ll::rocks_statistics_get_ticker_count(self.raw, mem::transmute(ticker_type))
        }
    }

    fn histogram_data(&self, type_: Histograms) -> HistogramData {
        unsafe {
            let mut data: HistogramData = mem::zeroed();
            ll::rocks_statistics_histogram_data(self.raw,
                                                mem::transmute(type_),
                                                &mut data as *mut HistogramData as *mut ll::rocks_histogram_data_t);
            data
        }
    }

    fn get_histogram_string(&self, type_: Histograms) -> String {
        let mut ret = String::new();
        unsafe {
            ll::rocks_statistics_get_histogram_string(self.raw,
                                                      mem::transmute(type_),
                                                      &mut ret as *mut String as *mut _);
        }
        ret
    }

    // add count to ticker
    fn record_tick(&mut self, ticker_type: Tickers, count: u64) {
        unsafe {
            ll::rocks_statistics_record_tick(self.raw, mem::transmute(ticker_type), count);
        }
    }

    fn set_ticker_count(&mut self, ticker_type: Tickers, count: u64) {
        unsafe {
            ll::rocks_statistics_set_ticker_count(self.raw, mem::transmute(ticker_type), count);
        }
    }

    fn get_and_reset_ticker_count(&mut self, ticker_type: Tickers) -> u64 {
        unsafe {
            ll::rocks_statistics_get_and_reset_ticker_count(self.raw, mem::transmute(ticker_type))
        }
    }

    fn measure_time(&mut self, histogram_type: Histograms, time: u64) {
        unsafe {
            ll::rocks_statistics_measure_time(self.raw, mem::transmute(histogram_type), time);
        }
    }

    // Override this function to disable particular histogram collection
    fn hist_enabled_for_type(&self, type_: Histograms) -> bool {
        unsafe {
            ll::rocks_statistics_hist_enabled_for_type(self.raw, mem::transmute(type_)) != 0
        }
    }
}

impl fmt::Display for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        unsafe {
            ll::rocks_statistics_to_string(self.raw,
                                           &mut s as *mut String as *mut c_void);
        }
        write!(f, "{}", s)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::rocksdb::*;
    use super::super::rate_limiter::RateLimiter;

    #[test]
    fn statistics() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();

        let stat = Statistics::new();

        let db = DB::open(Options::default()
                          .map_db_options(|db| {
                              db.create_if_missing(true)
                                  .statistics(Some(stat.clone())) // FIXME: is this the best way?
                                  .rate_limiter(Some(RateLimiter::new(1024, // 1 KiB/s
                                                                      10_000, // 10 ms
                                                                      10)))
                          }),
                          &tmp_dir)
            .unwrap();

        assert!(db.put(&Default::default(), b"long-key", vec![b'A'; 1024 * 1024].as_ref())
                .is_ok());
        assert!(db.put(&Default::default(), b"a", b"1").is_ok());
        assert!(db.put(&Default::default(), b"b", b"2").is_ok());
        assert!(db.put(&Default::default(), b"c", b"3").is_ok());

        assert!(db.compact_range(&Default::default(), ..).is_ok());

        assert!(db.get(&Default::default(), b"long-key").is_ok());

        println!("st => {}", stat);
        assert!(stat.get_ticker_count(Tickers::BlockCacheBytesWrite) > 0);
        // this is the last ticker, since we set up rate limiter to a low value, this must be true
        assert!(stat.get_ticker_count(Tickers::NumberRateLimiterDrains) > 0);

        // a multiline string
        assert!(stat.get_histogram_string(Histograms::BytesPerRead).len() > 100);
    }
}
