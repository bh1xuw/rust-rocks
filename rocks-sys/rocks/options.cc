#include <iostream>

#include "rocksdb/options.h"
#include "rocksdb/table.h"

#include "rocks/ctypes.hpp"


using namespace rocksdb;

using std::shared_ptr;

// rocks_options_t
extern "C" {

  rocks_options_t* rocks_options_create() {
    return new rocks_options_t;
  }

  void rocks_options_destroy(rocks_options_t* options) {
    delete options;
  }

  rocks_column_family_options_t* rocks_column_family_options_create() {
    return new rocks_column_family_options_t;
  }

  void rocks_column_family_options_destroy(rocks_column_family_options_t* options) {
    delete options;
  }

  void rocks_options_increase_parallelism(
                                          rocks_options_t* opt, int total_threads) {
    opt->rep.IncreaseParallelism(total_threads);
  }

  void rocks_options_optimize_for_point_lookup(
                                               rocks_options_t* opt, uint64_t block_cache_size_mb) {
    opt->rep.OptimizeForPointLookup(block_cache_size_mb);
  }

  void rocks_options_optimize_level_style_compaction(
                                                     rocks_options_t* opt, uint64_t memtable_memory_budget) {
    opt->rep.OptimizeLevelStyleCompaction(memtable_memory_budget);
  }

  void rocks_options_optimize_universal_style_compaction(
                                                         rocks_options_t* opt, uint64_t memtable_memory_budget) {
    opt->rep.OptimizeUniversalStyleCompaction(memtable_memory_budget);
  }

  /*
    void rocks_options_set_compaction_filter(
    rocks_options_t* opt,
    rocks_compactionfilter_t* filter) {
    opt->rep.compaction_filter = filter;
    }
    void rocks_options_set_compaction_filter_factory(
    rocks_options_t* opt, rocks_compactionfilterfactory_t* factory) {
    opt->rep.compaction_filter_factory =
    std::shared_ptr<CompactionFilterFactory>(factory);
    }
  */
  void rocks_options_compaction_readahead_size(
                                               rocks_options_t* opt, size_t s) {
    opt->rep.compaction_readahead_size = s;
  }

  /*
    void rocks_options_set_comparator(
    rocks_options_t* opt,
    rocks_comparator_t* cmp) {
    opt->rep.comparator = cmp;
    }
  */

  /*
    void rocks_options_set_merge_operator(
    rocks_options_t* opt,
    rocks_mergeoperator_t* merge_operator) {
    opt->rep.merge_operator = std::shared_ptr<MergeOperator>(merge_operator);
    }
  */

  void rocks_options_set_create_if_missing(
                                           rocks_options_t* opt, unsigned char v) {
    opt->rep.create_if_missing = v;
  }

  void rocks_options_set_create_missing_column_families(
                                                        rocks_options_t* opt, unsigned char v) {
    opt->rep.create_missing_column_families = v;
  }

  void rocks_options_set_error_if_exists(
                                         rocks_options_t* opt, unsigned char v) {
    opt->rep.error_if_exists = v;
  }

  void rocks_options_set_paranoid_checks(
                                         rocks_options_t* opt, unsigned char v) {
    opt->rep.paranoid_checks = v;
  }

  /*
    void rocks_options_set_env(rocks_options_t* opt, rocks_env_t* env) {
    opt->rep.env = (env ? env->rep : nullptr);
    }

    void rocks_options_set_info_log(rocks_options_t* opt, rocks_logger_t* l) {
    if (l) {
    opt->rep.info_log = l->rep;
    }
    }

    void rocks_options_set_info_log_level(
    rocks_options_t* opt, int v) {
    opt->rep.info_log_level = static_cast<InfoLogLevel>(v);
    }
  */

  void rocks_options_set_db_write_buffer_size(rocks_options_t* opt,
                                              size_t s) {
    opt->rep.db_write_buffer_size = s;
  }

  void rocks_options_set_write_buffer_size(rocks_options_t* opt, size_t s) {
    opt->rep.write_buffer_size = s;
  }

  void rocks_options_set_max_open_files(rocks_options_t* opt, int n) {
    opt->rep.max_open_files = n;
  }

  void rocks_options_set_max_total_wal_size(rocks_options_t* opt, uint64_t n) {
    opt->rep.max_total_wal_size = n;
  }

  void rocks_options_set_target_file_size_base(
                                               rocks_options_t* opt, uint64_t n) {
    opt->rep.target_file_size_base = n;
  }

  void rocks_options_set_target_file_size_multiplier(
                                                     rocks_options_t* opt, int n) {
    opt->rep.target_file_size_multiplier = n;
  }

  void rocks_options_set_max_bytes_for_level_base(
                                                  rocks_options_t* opt, uint64_t n) {
    opt->rep.max_bytes_for_level_base = n;
  }

  void rocks_options_set_level_compaction_dynamic_level_bytes(
                                                              rocks_options_t* opt, unsigned char v) {
    opt->rep.level_compaction_dynamic_level_bytes = v;
  }

  void rocks_options_set_max_bytes_for_level_multiplier(rocks_options_t* opt,
                                                        double n) {
    opt->rep.max_bytes_for_level_multiplier = n;
  }

  void rocks_options_set_max_compaction_bytes(rocks_options_t* opt,
                                              uint64_t n) {
    opt->rep.max_compaction_bytes = n;
  }

  void rocks_options_set_max_bytes_for_level_multiplier_additional(
                                                                   rocks_options_t* opt, int* level_values, size_t num_levels) {
    opt->rep.max_bytes_for_level_multiplier_additional.resize(num_levels);
    for (size_t i = 0; i < num_levels; ++i) {
      opt->rep.max_bytes_for_level_multiplier_additional[i] = level_values[i];
    }
  }

  void rocks_options_enable_statistics(rocks_options_t* opt) {
    opt->rep.statistics = rocksdb::CreateDBStatistics();
  }

  void rocks_options_set_num_levels(rocks_options_t* opt, int n) {
    opt->rep.num_levels = n;
  }

  void rocks_options_set_level0_file_num_compaction_trigger(
                                                            rocks_options_t* opt, int n) {
    opt->rep.level0_file_num_compaction_trigger = n;
  }

  void rocks_options_set_level0_slowdown_writes_trigger(
                                                        rocks_options_t* opt, int n) {
    opt->rep.level0_slowdown_writes_trigger = n;
  }

  void rocks_options_set_level0_stop_writes_trigger(
                                                    rocks_options_t* opt, int n) {
    opt->rep.level0_stop_writes_trigger = n;
  }

  void rocks_options_set_max_mem_compaction_level(rocks_options_t* opt,
                                                  int n) {}

  void rocks_options_set_wal_recovery_mode(rocks_options_t* opt,int mode) {
    opt->rep.wal_recovery_mode = static_cast<WALRecoveryMode>(mode);
  }

  void rocks_options_set_compression(rocks_options_t* opt, int t) {
    opt->rep.compression = static_cast<CompressionType>(t);
  }

  void rocks_options_set_compression_per_level(rocks_options_t* opt,
                                               int* level_values,
                                               size_t num_levels) {
    opt->rep.compression_per_level.resize(num_levels);
    for (size_t i = 0; i < num_levels; ++i) {
      opt->rep.compression_per_level[i] =
        static_cast<CompressionType>(level_values[i]);
    }
  }

  void rocks_options_set_compression_options(rocks_options_t* opt, int w_bits,
                                             int level, int strategy,
                                             int max_dict_bytes) {
    opt->rep.compression_opts.window_bits = w_bits;
    opt->rep.compression_opts.level = level;
    opt->rep.compression_opts.strategy = strategy;
    opt->rep.compression_opts.max_dict_bytes = max_dict_bytes;
  }

  /*
    void rocks_options_set_prefix_extractor(
    rocks_options_t* opt, rocks_slicetransform_t* prefix_extractor) {
    opt->rep.prefix_extractor.reset(prefix_extractor);
    }
  */

  void rocks_options_set_disable_data_sync(
                                           rocks_options_t* opt, int disable_data_sync) {
    opt->rep.disableDataSync = disable_data_sync;
  }

  void rocks_options_set_use_fsync(
                                   rocks_options_t* opt, int use_fsync) {
    opt->rep.use_fsync = use_fsync;
  }

  void rocks_options_set_db_log_dir(
                                    rocks_options_t* opt, const char* db_log_dir) {
    opt->rep.db_log_dir = db_log_dir;
  }

  void rocks_options_set_wal_dir(
                                 rocks_options_t* opt, const char* v) {
    opt->rep.wal_dir = v;
  }

  void rocks_options_set_WAL_ttl_seconds(rocks_options_t* opt, uint64_t ttl) {
    opt->rep.WAL_ttl_seconds = ttl;
  }

  void rocks_options_set_WAL_size_limit_MB(
                                           rocks_options_t* opt, uint64_t limit) {
    opt->rep.WAL_size_limit_MB = limit;
  }

  void rocks_options_set_manifest_preallocation_size(
                                                     rocks_options_t* opt, size_t v) {
    opt->rep.manifest_preallocation_size = v;
  }

  // noop
  void rocks_options_set_purge_redundant_kvs_while_flush(rocks_options_t* opt,
                                                         unsigned char v) {}

  void rocks_options_set_use_direct_reads(rocks_options_t* opt,
                                          unsigned char v) {
    opt->rep.use_direct_reads = v;
  }

  void rocks_options_set_use_direct_writes(rocks_options_t* opt,
                                           unsigned char v) {
    opt->rep.use_direct_writes = v;
  }

  void rocks_options_set_allow_mmap_reads(
                                          rocks_options_t* opt, unsigned char v) {
    opt->rep.allow_mmap_reads = v;
  }

  void rocks_options_set_allow_mmap_writes(
                                           rocks_options_t* opt, unsigned char v) {
    opt->rep.allow_mmap_writes = v;
  }

  void rocks_options_set_is_fd_close_on_exec(
                                             rocks_options_t* opt, unsigned char v) {
    opt->rep.is_fd_close_on_exec = v;
  }

  void rocks_options_set_skip_log_error_on_recovery(
                                                    rocks_options_t* opt, unsigned char v) {
    opt->rep.skip_log_error_on_recovery = v;
  }

  void rocks_options_set_stats_dump_period_sec(
                                               rocks_options_t* opt, unsigned int v) {
    opt->rep.stats_dump_period_sec = v;
  }

  void rocks_options_set_advise_random_on_open(
                                               rocks_options_t* opt, unsigned char v) {
    opt->rep.advise_random_on_open = v;
  }

  void rocks_options_set_access_hint_on_compaction_start(
                                                         rocks_options_t* opt, int v) {
    switch(v) {
    case 0:
      opt->rep.access_hint_on_compaction_start = rocksdb::Options::NONE;
      break;
    case 1:
      opt->rep.access_hint_on_compaction_start = rocksdb::Options::NORMAL;
      break;
    case 2:
      opt->rep.access_hint_on_compaction_start = rocksdb::Options::SEQUENTIAL;
      break;
    case 3:
      opt->rep.access_hint_on_compaction_start = rocksdb::Options::WILLNEED;
      break;
    }
  }

  void rocks_options_set_use_adaptive_mutex(
                                            rocks_options_t* opt, unsigned char v) {
    opt->rep.use_adaptive_mutex = v;
  }

  void rocks_options_set_bytes_per_sync(
                                        rocks_options_t* opt, uint64_t v) {
    opt->rep.bytes_per_sync = v;
  }

  void rocks_options_set_allow_concurrent_memtable_write(rocks_options_t* opt,
                                                         unsigned char v) {
    opt->rep.allow_concurrent_memtable_write = v;
  }

  void rocks_options_set_enable_write_thread_adaptive_yield(
                                                            rocks_options_t* opt, unsigned char v) {
    opt->rep.enable_write_thread_adaptive_yield = v;
  }

  void rocks_options_set_verify_checksums_in_compaction(
                                                        rocks_options_t* opt, unsigned char v) {
    opt->rep.verify_checksums_in_compaction = v;
  }

  void rocks_options_set_max_sequential_skip_in_iterations(
                                                           rocks_options_t* opt, uint64_t v) {
    opt->rep.max_sequential_skip_in_iterations = v;
  }

  void rocks_options_set_max_write_buffer_number(rocks_options_t* opt, int n) {
    opt->rep.max_write_buffer_number = n;
  }

  void rocks_options_set_min_write_buffer_number_to_merge(rocks_options_t* opt, int n) {
    opt->rep.min_write_buffer_number_to_merge = n;
  }

  void rocks_options_set_max_write_buffer_number_to_maintain(
                                                             rocks_options_t* opt, int n) {
    opt->rep.max_write_buffer_number_to_maintain = n;
  }

  void rocks_options_set_max_background_compactions(rocks_options_t* opt, int n) {
    opt->rep.max_background_compactions = n;
  }

  void rocks_options_set_base_background_compactions(rocks_options_t* opt,
                                                     int n) {
    opt->rep.base_background_compactions = n;
  }

  void rocks_options_set_max_background_flushes(rocks_options_t* opt, int n) {
    opt->rep.max_background_flushes = n;
  }

  void rocks_options_set_max_log_file_size(rocks_options_t* opt, size_t v) {
    opt->rep.max_log_file_size = v;
  }

  void rocks_options_set_log_file_time_to_roll(rocks_options_t* opt, size_t v) {
    opt->rep.log_file_time_to_roll = v;
  }

  void rocks_options_set_keep_log_file_num(rocks_options_t* opt, size_t v) {
    opt->rep.keep_log_file_num = v;
  }

  void rocks_options_set_recycle_log_file_num(rocks_options_t* opt,
                                              size_t v) {
    opt->rep.recycle_log_file_num = v;
  }

  void rocks_options_set_soft_rate_limit(rocks_options_t* opt, double v) {
    opt->rep.soft_rate_limit = v;
  }

  void rocks_options_set_hard_rate_limit(rocks_options_t* opt, double v) {
    opt->rep.hard_rate_limit = v;
  }

  void rocks_options_set_soft_pending_compaction_bytes_limit(rocks_options_t* opt, size_t v) {
    opt->rep.soft_pending_compaction_bytes_limit = v;
  }

  void rocks_options_set_hard_pending_compaction_bytes_limit(rocks_options_t* opt, size_t v) {
    opt->rep.hard_pending_compaction_bytes_limit = v;
  }

  void rocks_options_set_rate_limit_delay_max_milliseconds(
                                                           rocks_options_t* opt, unsigned int v) {
    opt->rep.rate_limit_delay_max_milliseconds = v;
  }

  void rocks_options_set_max_manifest_file_size(
                                                rocks_options_t* opt, size_t v) {
    opt->rep.max_manifest_file_size = v;
  }

  void rocks_options_set_table_cache_numshardbits(
                                                  rocks_options_t* opt, int v) {
    opt->rep.table_cache_numshardbits = v;
  }

  void rocks_options_set_table_cache_remove_scan_count_limit(
                                                             rocks_options_t* opt, int v) {
    // this option is deprecated
  }

  void rocks_options_set_arena_block_size(
                                          rocks_options_t* opt, size_t v) {
    opt->rep.arena_block_size = v;
  }

  void rocks_options_set_disable_auto_compactions(rocks_options_t* opt, int disable) {
    opt->rep.disable_auto_compactions = disable;
  }

  void rocks_options_set_optimize_filters_for_hits(rocks_options_t* opt, int v) {
    opt->rep.optimize_filters_for_hits = v;
  }

  void rocks_options_set_delete_obsolete_files_period_micros(
                                                             rocks_options_t* opt, uint64_t v) {
    opt->rep.delete_obsolete_files_period_micros = v;
  }

  void rocks_options_prepare_for_bulk_load(rocks_options_t* opt) {
    opt->rep.PrepareForBulkLoad();
  }

  void rocks_options_set_memtable_vector_rep(rocks_options_t *opt) {
    opt->rep.memtable_factory.reset(new rocksdb::VectorRepFactory);
  }

  void rocks_options_set_memtable_prefix_bloom_size_ratio(
                                                          rocks_options_t* opt, double v) {
    opt->rep.memtable_prefix_bloom_size_ratio = v;
  }

  void rocks_options_set_memtable_huge_page_size(rocks_options_t* opt,
                                                 size_t v) {
    opt->rep.memtable_huge_page_size = v;
  }

  /*
    void rocks_options_set_hash_skip_list_rep(
    rocks_options_t *opt, size_t bucket_count,
    int32_t skiplist_height, int32_t skiplist_branching_factor) {
    rocksdb::MemTableRepFactory* factory = rocksdb::NewHashSkipListRepFactory(
    bucket_count, skiplist_height, skiplist_branching_factor);
    opt->rep.memtable_factory.reset(factory);
    }
  */

  void rocks_options_set_hash_link_list_rep(
                                            rocks_options_t *opt, size_t bucket_count) {
    opt->rep.memtable_factory.reset(rocksdb::NewHashLinkListRepFactory(bucket_count));
  }

  void rocks_options_set_plain_table_factory(
                                             rocks_options_t *opt, uint32_t user_key_len, int bloom_bits_per_key,
                                             double hash_table_ratio, size_t index_sparseness) {
    rocksdb::PlainTableOptions options;
    options.user_key_len = user_key_len;
    options.bloom_bits_per_key = bloom_bits_per_key;
    options.hash_table_ratio = hash_table_ratio;
    options.index_sparseness = index_sparseness;

    rocksdb::TableFactory* factory = rocksdb::NewPlainTableFactory(options);
    opt->rep.table_factory.reset(factory);
  }

  void rocks_options_set_max_successive_merges(
                                               rocks_options_t* opt, size_t v) {
    opt->rep.max_successive_merges = v;
  }

  void rocks_options_set_min_partial_merge_operands(
                                                    rocks_options_t* opt, uint32_t v) {
    opt->rep.min_partial_merge_operands = v;
  }

  void rocks_options_set_bloom_locality(
                                        rocks_options_t* opt, uint32_t v) {
    opt->rep.bloom_locality = v;
  }

  void rocks_options_set_inplace_update_support(
                                                rocks_options_t* opt, unsigned char v) {
    opt->rep.inplace_update_support = v;
  }

  void rocks_options_set_inplace_update_num_locks(
                                                  rocks_options_t* opt, size_t v) {
    opt->rep.inplace_update_num_locks = v;
  }

  void rocks_options_set_report_bg_io_stats(
                                            rocks_options_t* opt, int v) {
    opt->rep.report_bg_io_stats = v;
  }

  void rocks_options_set_compaction_style(rocks_options_t *opt, int style) {
    opt->rep.compaction_style = static_cast<rocksdb::CompactionStyle>(style);
  }

  /*
    void rocks_options_set_universal_compaction_options(rocks_options_t *opt, rocks_universal_compaction_options_t *uco) {
    opt->rep.compaction_options_universal = *(uco->rep);
    }
    void rocks_options_set_fifo_compaction_options(
    rocks_options_t* opt,
    rocks_fifo_compaction_options_t* fifo) {
    opt->rep.compaction_options_fifo = fifo->rep;
    }

    char *rocks_options_statistics_get_string(rocks_options_t *opt) {
    rocksdb::Statistics *statistics = opt->rep.statistics.get();
    if (statistics) {
    return strdup(statistics->ToString().c_str());
    }
    return nullptr;
    }
  */

  void rocks_options_set_ratelimiter(rocks_options_t *opt, rocks_ratelimiter_t *limiter) {
    opt->rep.rate_limiter.reset(limiter->rep);
    limiter->rep = nullptr;
  }

}

// rocks_readoptions_t
extern "C" {
  rocks_readoptions_t* rocks_readoptions_create() {
    return new rocks_readoptions_t;
  }

  void rocks_readoptions_destroy(rocks_readoptions_t* opt) {
    delete opt;
  }

  void rocks_readoptions_set_verify_checksums(
                                              rocks_readoptions_t* opt,
                                              unsigned char v) {
    opt->rep.verify_checksums = v;
  }

  void rocks_readoptions_set_fill_cache(
                                        rocks_readoptions_t* opt, unsigned char v) {
    opt->rep.fill_cache = v;
  }

  void rocks_readoptions_set_snapshot(
                                      rocks_readoptions_t* opt,
                                      const rocks_snapshot_t* snap) {
    opt->rep.snapshot = (snap ? snap->rep : nullptr);
  }

  void rocks_readoptions_set_iterate_upper_bound(
                                                 rocks_readoptions_t* opt,
                                                 const char* key, size_t keylen) {
    if (key == nullptr) {
      opt->upper_bound = Slice();
      opt->rep.iterate_upper_bound = nullptr;

    } else {
      opt->upper_bound = Slice(key, keylen);
      opt->rep.iterate_upper_bound = &opt->upper_bound;
    }
  }

  void rocks_readoptions_set_read_tier(
                                       rocks_readoptions_t* opt, int v) {
    opt->rep.read_tier = static_cast<rocksdb::ReadTier>(v);
  }

  void rocks_readoptions_set_tailing(
                                     rocks_readoptions_t* opt, unsigned char v) {
    opt->rep.tailing = v;
  }

  void rocks_readoptions_set_readahead_size(
                                            rocks_readoptions_t* opt, size_t v) {
    opt->rep.readahead_size = v;
  }

  void rocks_readoptions_set_pin_data(rocks_readoptions_t* opt,
                                      unsigned char v) {
    opt->rep.pin_data = v;
  }

  void rocks_readoptions_set_total_order_seek(rocks_readoptions_t* opt,
                                              unsigned char v) {
    opt->rep.total_order_seek = v;
  }
}

extern "C" {
  rocks_writeoptions_t* rocks_writeoptions_create() {
    return new rocks_writeoptions_t;
  }

  void rocks_writeoptions_destroy(rocks_writeoptions_t* opt) {
    delete opt;
  }

  void rocks_writeoptions_set_sync(
                                   rocks_writeoptions_t* opt, unsigned char v) {
    opt->rep.sync = v;
  }

  void rocks_writeoptions_disable_WAL(rocks_writeoptions_t* opt, int disable) {
    opt->rep.disableWAL = disable;
  }
}


extern "C" {
  rocks_logger_t *rocks_create_logger_from_options(const char *path,
                                                   rocks_options_t *opts,
                                                   rocks_status_t *status) {
    rocks_logger_t *logger = new rocks_logger_t;
    Status st = CreateLoggerFromOptions(std::string(path), opts->rep,
                                        &logger->rep);
    rocks_status_convert(&st, status);
    if (!st.ok()) {
      delete logger;
      return nullptr;
    }
    return logger;
  }
}
