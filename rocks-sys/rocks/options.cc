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

  rocks_dboptions_t* rocks_dboptions_create() {
    return new rocks_dboptions_t;
  }

  void rocks_dboptions_destroy(rocks_dboptions_t* options) {
    delete options;
  }

  rocks_cfoptions_t* rocks_cfoptions_create() {
    return new rocks_cfoptions_t;
  }

  void rocks_cfoptions_destroy(rocks_cfoptions_t* options) {
    delete options;
  }

  // upconvert, downconvert
  rocks_options_t* rocks_options_create_from_db_cf_options(rocks_dboptions_t* dbopt, rocks_cfoptions_t* cfopt) {
    return new rocks_options_t { Options(dbopt->rep, cfopt->rep) };
  }

  rocks_dboptions_t* rocks_dboptions_create_from_options(rocks_options_t* options) {
    return new rocks_dboptions_t { DBOptions(options->rep) };
  }

  rocks_cfoptions_t* rocks_cfoptions_create_from_options(rocks_options_t* options) {
    return new rocks_cfoptions_t { ColumnFamilyOptions(options->rep) };
  }

  // cfoptions

  void rocks_cfoptions_optimize_for_small_db(rocks_cfoptions_t* opt) {
    opt->rep.OptimizeForSmallDb();
  }

  void rocks_cfoptions_optimize_for_point_lookup(rocks_cfoptions_t* opt, uint64_t block_cache_size_mb) {
    opt->rep.OptimizeForPointLookup(block_cache_size_mb);
  }

  void rocks_cfoptions_optimize_level_style_compaction(rocks_cfoptions_t* opt, uint64_t memtable_memory_budget) {
    opt->rep.OptimizeLevelStyleCompaction(memtable_memory_budget);
  }

  void rocks_cfoptions_optimize_universal_style_compaction(rocks_cfoptions_t* opt, uint64_t memtable_memory_budget) {
    opt->rep.OptimizeUniversalStyleCompaction(memtable_memory_budget);
  }

  void rocks_cfoptions_set_comparator_by_trait(rocks_cfoptions_t* opt, void* cp_trait_obj) {
    // FIXME: mem leak
    opt->rep.comparator = new rocks_comparator_t { cp_trait_obj };
  }

  void rocks_cfoptions_set_bitwise_comparator(rocks_cfoptions_t* opt, unsigned char reversed) {
    if (reversed) {
      opt->rep.comparator = ReverseBytewiseComparator();
    } else {
      // this is default
      opt->rep.comparator = BytewiseComparator();
    }
  }

  void rocks_cfoptions_set_merge_operator_by_assoc_op_trait(rocks_cfoptions_t* opt, void* op_trait_obj) {
    opt->rep.merge_operator = std::shared_ptr<MergeOperator>(
                                                             new rocks_associative_mergeoperator_t { op_trait_obj }
                                                             );
  }

  void rocks_cfoptions_set_merge_operator_by_merge_op_trait(rocks_cfoptions_t* opt, void* op_trait_obj) {
    opt->rep.merge_operator = std::shared_ptr<MergeOperator>(
                                                             new rocks_mergeoperator_t { op_trait_obj }
                                                             );
  }

  // FIXME: mem leaks?
  void rocks_cfoptions_set_compaction_filter_by_trait(rocks_cfoptions_t* opt, void* filter_trait_obj) {
    opt->rep.compaction_filter = new rocks_compaction_filter_t { filter_trait_obj };
  }
  /*
  void rocks_cfoptions_set_compaction_filter_factory(
                                                   rocks_options_t* opt, rocks_compactionfilterfactory_t* factory) {
    opt->rep.compaction_filter_factory =
      std::shared_ptr<CompactionFilterFactory>(factory);
  }
  */

  void rocks_cfoptions_set_write_buffer_size(rocks_cfoptions_t* opt, size_t s) {
    opt->rep.write_buffer_size = s;
  }

  void rocks_cfoptions_set_compression(rocks_cfoptions_t* opt, int t) {
    opt->rep.compression = static_cast<CompressionType>(t);
  }

  void rocks_cfoptions_set_bottommost_compression(rocks_cfoptions_t* opt, int t) {
    opt->rep.bottommost_compression = static_cast<CompressionType>(t);
  }

  void rocks_cfoptions_set_compression_options(rocks_cfoptions_t* opt, int w_bits,
                                               int level, int strategy,
                                               uint32_t max_dict_bytes) {
    opt->rep.compression_opts.window_bits = w_bits;
    opt->rep.compression_opts.level = level;
    opt->rep.compression_opts.strategy = strategy;
    opt->rep.compression_opts.max_dict_bytes = max_dict_bytes;
  }

  void rocks_cfoptions_set_level0_file_num_compaction_trigger(rocks_cfoptions_t* opt, int n) {
    opt->rep.level0_file_num_compaction_trigger = n;
  }

  void rocks_cfoptions_set_prefix_extractor_by_trait(rocks_cfoptions_t* opt, void* trans_trait_obj) {
    opt->rep.prefix_extractor.reset(new rocks_slice_transform_t { trans_trait_obj });
  }

  void rocks_cfoptions_set_prefix_extractor_fixed_prefix(rocks_cfoptions_t* opt, size_t prefix_len) {
    opt->rep.prefix_extractor.reset(NewFixedPrefixTransform(prefix_len));
  }

  void rocks_cfoptions_set_prefix_extractor_capped_prefix(rocks_cfoptions_t* opt, size_t cap_len) {
    opt->rep.prefix_extractor.reset(NewCappedPrefixTransform(cap_len));
  }

  void rocks_cfoptions_set_prefix_extractor_noop(rocks_cfoptions_t* opt) {
    opt->rep.prefix_extractor.reset(NewNoopTransform());
  }

  void rocks_cfoptions_set_max_bytes_for_level_base(rocks_cfoptions_t* opt, uint64_t n) {
    opt->rep.max_bytes_for_level_base = n;
  }

  void rocks_cfoptions_set_disable_auto_compactions(rocks_cfoptions_t* opt, unsigned char disable) {
    opt->rep.disable_auto_compactions = disable;
  }

  // rocks_cfoptions_set_table_factory()
  // table_factory
  void rocks_cfoptions_set_plain_table_factory(
                                             rocks_cfoptions_t *opt, uint32_t user_key_len, int bloom_bits_per_key,
                                             double hash_table_ratio, size_t index_sparseness) {
    rocksdb::PlainTableOptions options;
    options.user_key_len = user_key_len;
    options.bloom_bits_per_key = bloom_bits_per_key;
    options.hash_table_ratio = hash_table_ratio;
    options.index_sparseness = index_sparseness;

    rocksdb::TableFactory* factory = rocksdb::NewPlainTableFactory(options);
    opt->rep.table_factory.reset(factory);
  }

  // via AdvancedColumnFamilyOptions

  void rocks_cfoptions_set_max_write_buffer_number(rocks_cfoptions_t* opt, int n) {
    opt->rep.max_write_buffer_number = n;
  }

  void rocks_cfoptions_set_min_write_buffer_number_to_merge(rocks_cfoptions_t* opt, int n) {
    opt->rep.min_write_buffer_number_to_merge = n;
  }

  void rocks_cfoptions_set_max_write_buffer_number_to_maintain(
                                                             rocks_cfoptions_t* opt, int n) {
    opt->rep.max_write_buffer_number_to_maintain = n;
  }

  void rocks_cfoptions_set_inplace_update_support(
                                                rocks_cfoptions_t* opt, unsigned char v) {
    opt->rep.inplace_update_support = v;
  }

  void rocks_cfoptions_set_inplace_update_num_locks(
                                                  rocks_cfoptions_t* opt, size_t v) {
    opt->rep.inplace_update_num_locks = v;
  }

  // inplace_callback

  void rocks_cfoptions_set_memtable_prefix_bloom_size_ratio(
                                                          rocks_cfoptions_t* opt, double v) {
    opt->rep.memtable_prefix_bloom_size_ratio = v;
  }

  void rocks_cfoptions_set_memtable_huge_page_size(rocks_cfoptions_t* opt,
                                                 size_t v) {
    opt->rep.memtable_huge_page_size = v;
  }

  // TODO: fix this style
  void rocks_cfoptions_set_memtable_insert_with_hint_prefix_extractor_by_trait(rocks_cfoptions_t* opt, void* trans_trait_obj) {
    opt->rep.memtable_insert_with_hint_prefix_extractor.reset(new rocks_slice_transform_t { trans_trait_obj });
  }
  void rocks_cfoptions_set_memtable_insert_with_hint_prefix_extractor_fixed_prefix(rocks_cfoptions_t* opt, size_t prefix_len) {
    opt->rep.memtable_insert_with_hint_prefix_extractor.reset(NewFixedPrefixTransform(prefix_len));
  }
  void rocks_cfoptions_set_memtable_insert_with_hint_prefix_extractor_capped_prefix(rocks_cfoptions_t* opt, size_t cap_len) {
    opt->rep.memtable_insert_with_hint_prefix_extractor.reset(NewCappedPrefixTransform(cap_len));
  }
  void rocks_cfoptions_set_memtable_insert_with_hint_prefix_extractor_noop(rocks_cfoptions_t* opt) {
    opt->rep.memtable_insert_with_hint_prefix_extractor.reset(NewNoopTransform());
  }

  void rocks_cfoptions_set_bloom_locality(
                                        rocks_cfoptions_t* opt, uint32_t v) {
    opt->rep.bloom_locality = v;
  }

  void rocks_cfoptions_set_arena_block_size(
                                          rocks_cfoptions_t* opt, size_t v) {
    opt->rep.arena_block_size = v;
  }

  void rocks_cfoptions_set_compression_per_level(rocks_cfoptions_t* opt,
                                                 const int* level_values,
                                                 size_t num_levels) {
    opt->rep.compression_per_level.resize(num_levels);
    for (size_t i = 0; i < num_levels; ++i) {
      opt->rep.compression_per_level[i] =
        static_cast<CompressionType>(level_values[i]);
    }
  }

  void rocks_cfoptions_set_num_levels(rocks_cfoptions_t* opt, int n) {
    opt->rep.num_levels = n;
  }

  void rocks_cfoptions_set_level0_slowdown_writes_trigger(
                                                        rocks_cfoptions_t* opt, int n) {
    opt->rep.level0_slowdown_writes_trigger = n;
  }

  void rocks_cfoptions_set_level0_stop_writes_trigger(
                                                    rocks_cfoptions_t* opt, int n) {
    opt->rep.level0_stop_writes_trigger = n;
  }

  void rocks_cfoptions_set_target_file_size_base(
                                               rocks_cfoptions_t* opt, uint64_t n) {
    opt->rep.target_file_size_base = n;
  }

  void rocks_cfoptions_set_target_file_size_multiplier(
                                                     rocks_cfoptions_t* opt, int n) {
    opt->rep.target_file_size_multiplier = n;
  }

  void rocks_cfoptions_set_level_compaction_dynamic_level_bytes(
                                                              rocks_cfoptions_t* opt, unsigned char v) {
    opt->rep.level_compaction_dynamic_level_bytes = v;
  }

  void rocks_cfoptions_set_max_bytes_for_level_multiplier(rocks_cfoptions_t* opt,
                                                        double n) {
    opt->rep.max_bytes_for_level_multiplier = n;
  }

  void rocks_cfoptions_set_max_bytes_for_level_multiplier_additional(
                                                                   rocks_cfoptions_t* opt, int* level_values, size_t num_levels) {
    opt->rep.max_bytes_for_level_multiplier_additional.resize(num_levels);
    for (size_t i = 0; i < num_levels; ++i) {
      opt->rep.max_bytes_for_level_multiplier_additional[i] = level_values[i];
    }
  }

  void rocks_cfoptions_set_max_compaction_bytes(rocks_cfoptions_t* opt,
                                              uint64_t n) {
    opt->rep.max_compaction_bytes = n;
  }

  void rocks_cfoptions_set_soft_pending_compaction_bytes_limit(rocks_cfoptions_t* opt, uint64_t v) {
    opt->rep.soft_pending_compaction_bytes_limit = v;
  }

  void rocks_cfoptions_set_hard_pending_compaction_bytes_limit(rocks_cfoptions_t* opt, uint64_t v) {
    opt->rep.hard_pending_compaction_bytes_limit = v;
  }

  void rocks_cfoptions_set_compaction_style(rocks_cfoptions_t *opt, int style) {
    opt->rep.compaction_style = static_cast<rocksdb::CompactionStyle>(style);
  }

  void rocks_cfoptions_set_compaction_pri(rocks_cfoptions_t *opt, int pri) {
    opt->rep.compaction_pri = static_cast<rocksdb::CompactionPri>(pri);
  }

  /*
    void rocks_cfoptions_set_universal_compaction_options(rocks_cfoptions_t *opt, rocks_universal_compaction_options_t *uco) {
    opt->rep.compaction_options_universal = *(uco->rep);
  }
  */

  /*
  void rocks_cfoptions_set_fifo_compaction_options(
                                                 rocks_cfoptions_t* opt,
                                                 rocks_fifo_compaction_options_t* fifo) {
    opt->rep.compaction_options_fifo = fifo->rep;
  }
  */

  void rocks_cfoptions_set_max_sequential_skip_in_iterations(rocks_cfoptions_t* opt, uint64_t v) {
    opt->rep.max_sequential_skip_in_iterations = v;
  }

  // memtable_factory
  void rocks_cfoptions_set_memtable_vector_rep(rocks_cfoptions_t *opt) {
    opt->rep.memtable_factory.reset(new rocksdb::VectorRepFactory);
  }

  void rocks_cfoptions_set_hash_skip_list_rep(
    rocks_cfoptions_t *opt, size_t bucket_count,
    int32_t skiplist_height, int32_t skiplist_branching_factor) {
    rocksdb::MemTableRepFactory* factory = rocksdb::NewHashSkipListRepFactory(
    bucket_count, skiplist_height, skiplist_branching_factor);
    opt->rep.memtable_factory.reset(factory);
  }

  void rocks_cfoptions_set_hash_link_list_rep(rocks_cfoptions_t *opt, size_t bucket_count) {
    opt->rep.memtable_factory.reset(rocksdb::NewHashLinkListRepFactory(bucket_count));
  }

  /*
  void rocks_cfoptions_set_table_properties_collector_factories(rocks_cfoptions_t *opt,
                                                                rocks_table_properties_collector_factory_t* factories,
                                                                size_t n) {
  }
  */

  void rocks_cfoptions_set_max_successive_merges(rocks_cfoptions_t* opt, size_t v) {
    opt->rep.max_successive_merges = v;
  }

  void rocks_cfoptions_set_optimize_filters_for_hits(rocks_cfoptions_t* opt, unsigned char v) {
    opt->rep.optimize_filters_for_hits = v;
  }

  void rocks_cfoptions_set_paranoid_file_checks(rocks_cfoptions_t* opt, unsigned char v) {
    opt->rep.paranoid_file_checks = v;
  }

  void rocks_cfoptions_set_force_consistency_checks(rocks_cfoptions_t* opt, unsigned char v) {
    opt->rep.force_consistency_checks = v;;
  }

  void rocks_cfoptions_set_report_bg_io_stats(rocks_cfoptions_t* opt, unsigned char v) {
    opt->rep.report_bg_io_stats = v;
  }

  // dboptions

  void rocks_dboptions_optimize_for_small_db(rocks_dboptions_t* opt) {
    opt->rep.OptimizeForSmallDb();
  }

  void rocks_dboptions_increase_parallelism(rocks_dboptions_t* opt, int total_threads) {
    opt->rep.IncreaseParallelism(total_threads);
  }

  void rocks_dboptions_set_create_if_missing(
                                           rocks_dboptions_t* opt, unsigned char v) {
    opt->rep.create_if_missing = v;
  }

  void rocks_dboptions_set_create_missing_column_families(
                                                        rocks_dboptions_t* opt, unsigned char v) {
    opt->rep.create_missing_column_families = v;
  }

  void rocks_dboptions_set_error_if_exists(
                                         rocks_dboptions_t* opt, unsigned char v) {
    opt->rep.error_if_exists = v;
  }

  void rocks_dboptions_set_paranoid_checks(rocks_dboptions_t* opt, unsigned char v) {
    opt->rep.paranoid_checks = v;
  }

  void rocks_dboptions_set_env(rocks_dboptions_t* opt, rocks_env_t* env) {
    opt->rep.env = (env ? env->rep : nullptr);
  }

  void rocks_dboptions_set_ratelimiter(rocks_dboptions_t *opt, rocks_ratelimiter_t *limiter) {
    if (limiter != nullptr) {
      opt->rep.rate_limiter.reset(limiter->rep);
    } else {
      opt->rep.rate_limiter.reset((RateLimiter*)nullptr);
    }
  }

  // sst_file_manager
  /*
  void rocks_dboptions_set_sst_file_manager(rocks_dboptions_t* opt, rocks_sst_file_manager_t* manager) {
    opt->rep.sst_file_manager = manager->rep;
  }
  */

  void rocks_dboptions_set_info_log(rocks_dboptions_t* opt, rocks_logger_t* l) {
    if (l) {
      opt->rep.info_log = l->rep;
    }
  }

  void rocks_dboptions_set_info_log_level(rocks_dboptions_t* opt, int v) {
    opt->rep.info_log_level = static_cast<InfoLogLevel>(v);
  }

  void rocks_dboptions_set_max_open_files(rocks_dboptions_t* opt, int n) {
    opt->rep.max_open_files = n;
  }

  void rocks_dboptions_set_max_file_opening_threads(rocks_dboptions_t* opt, int n) {
    opt->rep.max_file_opening_threads = n;
  }

  void rocks_dboptions_set_max_total_wal_size(rocks_dboptions_t* opt, uint64_t n) {
    opt->rep.max_total_wal_size = n;
  }

  void rocks_dboptions_enable_statistics(rocks_dboptions_t* opt) {
    opt->rep.statistics = rocksdb::CreateDBStatistics();
  }

  void rocks_dboptions_set_use_fsync(
                                   rocks_dboptions_t* opt, unsigned char use_fsync) {
    opt->rep.use_fsync = use_fsync;
  }

  void rocks_dboptions_set_db_paths(rocks_dboptions_t* opt,
                                    const char* const* paths,
                                    const size_t* path_lens,
                                    const uint64_t* target_sizes,
                                    int size) {
    std::vector<DbPath> dbpaths;
    for (int i = 0; i < size; i++) {
      dbpaths.push_back(DbPath(std::string(paths[i], path_lens[i]), target_sizes[i]));
    }
    opt->rep.db_paths = dbpaths;
  }

  void rocks_dboptions_set_db_log_dir(rocks_dboptions_t* opt, const char* db_log_dir, size_t len) {
    opt->rep.db_log_dir = std::string(db_log_dir, len);
  }

  void rocks_dboptions_set_wal_dir(rocks_dboptions_t* opt, const char* v, size_t len) {
    opt->rep.wal_dir = std::string(v, len);
  }

  void rocks_dboptions_set_delete_obsolete_files_period_micros(rocks_dboptions_t* opt, uint64_t v) {
    opt->rep.delete_obsolete_files_period_micros = v;
  }
  void rocks_dboptions_set_base_background_compactions(rocks_dboptions_t* opt, int n) {
    opt->rep.base_background_compactions = n;
  }

  void rocks_dboptions_set_max_background_compactions(rocks_dboptions_t* opt, int n) {
    opt->rep.max_background_compactions = n;
  }

  void rocks_dboptions_set_max_subcompactions(rocks_dboptions_t* opt, uint32_t n) {
    opt->rep.max_subcompactions = n;
  }

  void rocks_dboptions_set_max_background_flushes(rocks_dboptions_t* opt, int n) {
    opt->rep.max_background_flushes = n;
  }

  void rocks_dboptions_set_max_log_file_size(rocks_dboptions_t* opt, size_t v) {
    opt->rep.max_log_file_size = v;
  }

  void rocks_dboptions_set_log_file_time_to_roll(rocks_dboptions_t* opt, size_t v) {
    opt->rep.log_file_time_to_roll = v;
  }

  void rocks_dboptions_set_keep_log_file_num(rocks_dboptions_t* opt, size_t v) {
    opt->rep.keep_log_file_num = v;
  }

  void rocks_dboptions_set_recycle_log_file_num(rocks_dboptions_t* opt, size_t v) {
    opt->rep.recycle_log_file_num = v;
  }

  void rocks_dboptions_set_max_manifest_file_size(rocks_dboptions_t* opt, uint64_t v) {
    opt->rep.max_manifest_file_size = v;
  }

  void rocks_dboptions_set_table_cache_numshardbits(
                                                  rocks_dboptions_t* opt, int v) {
    opt->rep.table_cache_numshardbits = v;
  }

  void rocks_dboptions_set_wal_ttl_seconds(rocks_dboptions_t* opt, uint64_t ttl) {
    opt->rep.WAL_ttl_seconds = ttl;
  }

  void rocks_dboptions_set_wal_size_limit_mb(
                                           rocks_dboptions_t* opt, uint64_t limit) {
    opt->rep.WAL_size_limit_MB = limit;
  }

  void rocks_dboptions_set_manifest_preallocation_size(
                                                     rocks_dboptions_t* opt, size_t v) {
    opt->rep.manifest_preallocation_size = v;
  }

  void rocks_dboptions_set_allow_mmap_reads(
                                             rocks_dboptions_t* opt, unsigned char v) {
    opt->rep.allow_mmap_reads = v;
  }

  void rocks_dboptions_set_allow_mmap_writes(
                                           rocks_dboptions_t* opt, unsigned char v) {
    opt->rep.allow_mmap_writes = v;
  }

  void rocks_dboptions_set_use_direct_reads(rocks_dboptions_t* opt,
                                            unsigned char v) {
    opt->rep.use_direct_reads = v;
  }

  void rocks_dboptions_set_use_direct_writes(rocks_dboptions_t* opt,
                                             unsigned char v) {
    opt->rep.use_direct_writes = v;
  }

  void rocks_dboptions_set_allow_fallocate(rocks_dboptions_t* opt,
                                           unsigned char v) {
    opt->rep.allow_fallocate = v;
  }

  void rocks_dboptions_set_is_fd_close_on_exec(
                                             rocks_dboptions_t* opt, unsigned char v) {
    opt->rep.is_fd_close_on_exec = v;
  }

  void rocks_dboptions_set_stats_dump_period_sec(
                                               rocks_dboptions_t* opt, unsigned int v) {
    opt->rep.stats_dump_period_sec = v;
  }

  void rocks_dboptions_set_advise_random_on_open(
                                               rocks_dboptions_t* opt, unsigned char v) {
    opt->rep.advise_random_on_open = v;
  }

  void rocks_dboptions_set_db_write_buffer_size(rocks_dboptions_t* opt,
                                              size_t s) {
    opt->rep.db_write_buffer_size = s;
  }

  // write_buffer_manager
  /*
  void rocks_dboptions_set_write_buffer_manager(rocks_dboptions_t* opt,
                                              rocks_write_buffer_manager_t* manager) {
    opt->rep.write_buffer_manager = manager->rep;
  }
  */

  void rocks_dboptions_set_access_hint_on_compaction_start(rocks_dboptions_t* opt, int v) {
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

  void rocks_dboptions_set_new_table_reader_for_compaction_inputs(
                                               rocks_dboptions_t* opt, unsigned char v) {
    opt->rep.new_table_reader_for_compaction_inputs = v;
  }

  void rocks_dboptions_set_compaction_readahead_size(
                                               rocks_dboptions_t* opt, size_t s) {
    opt->rep.compaction_readahead_size = s;
  }

  void rocks_dboptions_set_random_access_max_buffer_size(rocks_dboptions_t* opt,
                                              size_t s) {
    opt->rep.random_access_max_buffer_size = s;
  }

  void rocks_dboptions_set_writable_file_max_buffer_size(rocks_dboptions_t* opt,
                                                       size_t s) {
    opt->rep.writable_file_max_buffer_size = s;
  }

  void rocks_dboptions_set_use_adaptive_mutex(
                                            rocks_dboptions_t* opt, unsigned char v) {
    opt->rep.use_adaptive_mutex = v;
  }

  void rocks_dboptions_set_bytes_per_sync(
                                        rocks_dboptions_t* opt, uint64_t v) {
    opt->rep.bytes_per_sync = v;
  }

  void rocks_dboptions_set_wal_bytes_per_sync(
                                        rocks_dboptions_t* opt, uint64_t v) {
    opt->rep.wal_bytes_per_sync = v;
  }

  /*
  void rocks_dboptions_set_listeners(rocks_dboptions_t* opt, rocks_event_listener_t* listeners, size_t n) {
    //    opt->listerns = 
  }
  */

  void rocks_dboptions_set_enable_thread_tracking(
                                            rocks_dboptions_t* opt, unsigned char v) {
    opt->rep.enable_thread_tracking = v;
  }

  void rocks_dboptions_set_delayed_write_rate(
                                            rocks_dboptions_t* opt, uint64_t v) {
    opt->rep.delayed_write_rate = v;
  }

  void rocks_dboptions_set_allow_concurrent_memtable_write(rocks_dboptions_t* opt,
                                                         unsigned char v) {
    opt->rep.allow_concurrent_memtable_write = v;
  }

  void rocks_dboptions_set_enable_write_thread_adaptive_yield(
                                                            rocks_dboptions_t* opt, unsigned char v) {
    opt->rep.enable_write_thread_adaptive_yield = v;
  }

  void rocks_dboptions_set_write_thread_max_yield_usec(rocks_dboptions_t* opt, uint64_t v) {
    opt->rep.write_thread_max_yield_usec = v;
  }

  void rocks_dboptions_set_write_thread_slow_yield_usec(rocks_dboptions_t* opt, uint64_t v) {
    opt->rep.write_thread_slow_yield_usec = v;
  }

  void rocks_dboptions_set_skip_stats_update_on_db_open(rocks_dboptions_t* opt, unsigned char v) {
    opt->rep.skip_stats_update_on_db_open = v;
  }

  void rocks_dboptions_set_wal_recovery_mode(rocks_dboptions_t* opt, int mode) {
    opt->rep.wal_recovery_mode = static_cast<WALRecoveryMode>(mode);
  }

  void rocks_dboptions_set_allow_2pc(rocks_dboptions_t* opt, unsigned char v) {
    opt->rep.allow_2pc = v;
  }

  // FIXME: mem leaks?
  void rocks_dboptions_set_row_cache(rocks_dboptions_t* opt, rocks_cache_t* cache) {
    opt->rep.row_cache = cache->rep;
  }

  /*
  void rocks_dboptions_set_wal_filter(rocks_dboptions_t* opt, rocks_wal_filter_t* filter) {
    opt->rep.wal_filter = filter->rep;
  }
  */

  void rocks_dboptions_set_fail_if_options_file_error(rocks_dboptions_t* opt, unsigned char v) {
    opt->rep.fail_if_options_file_error = v;
  }

  void rocks_dboptions_set_dump_malloc_stats(rocks_dboptions_t* opt, unsigned char v) {
    opt->rep.dump_malloc_stats = v;
  }

  void rocks_dboptions_set_avoid_flush_during_recovery(rocks_dboptions_t* opt, unsigned char v) {
    opt->rep.avoid_flush_during_recovery = v;
  }

  void rocks_dboptions_set_avoid_flush_during_shutdown(rocks_dboptions_t* opt, unsigned char v) {
    opt->rep.avoid_flush_during_shutdown = v;
  }



  // opt

  void rocks_options_prepare_for_bulk_load(rocks_options_t* opt) {
    opt->rep.PrepareForBulkLoad();
  }

  void rocks_options_optimize_for_small_db(rocks_options_t* opt) {
    opt->rep.OptimizeForSmallDb();
  }

  /*
    char *rocks_options_statistics_get_string(rocks_options_t *opt) {
    rocksdb::Statistics *statistics = opt->rep.statistics.get();
    if (statistics) {
    return strdup(statistics->ToString().c_str());
    }
    return nullptr;
    }
  */

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

  void rocks_readoptions_set_managed(
                                     rocks_readoptions_t* opt, unsigned char v) {
    opt->rep.managed = v;
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

  void rocks_readoptions_set_prefix_same_as_start(rocks_readoptions_t* opt,
                                                  unsigned char v) {
    opt->rep.prefix_same_as_start = v;
  }

  void rocks_readoptions_set_ignore_range_deletions(rocks_readoptions_t* opt,
                                                    unsigned char v) {
    opt->rep.ignore_range_deletions = v;
  }

  void rocks_readoptions_set_background_purge_on_iterator_cleanup(rocks_readoptions_t* opt,
                                                                  unsigned char v) {
    opt->rep.background_purge_on_iterator_cleanup = v;
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

  void rocks_writeoptions_set_disable_wal(rocks_writeoptions_t* opt, unsigned char v) {
    opt->rep.disableWAL = v;
  }

  void rocks_writeoptions_set_ignore_missing_column_families(rocks_writeoptions_t* opt, unsigned char v) {
    opt->rep.ignore_missing_column_families = v;
  }

  void rocks_writeoptions_set_no_slowdown(rocks_writeoptions_t* opt, unsigned char v) {
    opt->rep.no_slowdown = v;
  }
}


extern "C" {
  rocks_compactrange_options_t* rocks_compactrange_options_create() {
    return new rocks_compactrange_options_t;
  }

  void rocks_compactrange_options_destroy(rocks_compactrange_options_t* opt) {
    delete opt;
  }

  void rocks_compactrange_options_set_exclusive_manual_compaction(
                                   rocks_compactrange_options_t* opt, unsigned char v) {
    opt->rep.exclusive_manual_compaction = v;
  }

  void rocks_compactrange_options_set_change_level(rocks_compactrange_options_t* opt, unsigned char v) {
    opt->rep.change_level = v;
  }

  void rocks_compactrange_options_set_target_level(rocks_compactrange_options_t* opt, int32_t v) {
    opt->rep.target_level = v;
  }

  void rocks_compactrange_options_set_target_path_id(rocks_compactrange_options_t* opt, uint32_t v) {
    opt->rep.target_path_id = v;
  }

  void rocks_compactrange_options_set_bottommost_level_compaction(rocks_compactrange_options_t* opt, int v) {
    opt->rep.bottommost_level_compaction = static_cast<BottommostLevelCompaction>(v);
  }
}

extern "C" {
  rocks_ingestexternalfile_options_t* rocks_ingestexternalfile_options_create() {
    return new rocks_ingestexternalfile_options_t;
  }

  void rocks_ingestexternalfile_options_destroy(rocks_ingestexternalfile_options_t* opt) {
    delete opt;
  }

  void rocks_ingestexternalfile_options_set_move_files(rocks_ingestexternalfile_options_t* opt, unsigned char v) {
    opt->rep.move_files = v;
  }
  void rocks_ingestexternalfile_options_set_snapshot_consistency(rocks_ingestexternalfile_options_t* opt, unsigned char v) {
    opt->rep.snapshot_consistency = v;
  }
  void rocks_ingestexternalfile_options_set_allow_global_seqno(rocks_ingestexternalfile_options_t* opt, unsigned char v) {
    opt->rep.allow_global_seqno = v;
  }
  void rocks_ingestexternalfile_options_set_allow_blocking_flush(rocks_ingestexternalfile_options_t* opt, unsigned char v) {
    opt->rep.allow_blocking_flush = v;
  }
}

extern "C" {
  rocks_flushoptions_t* rocks_flushoptions_create() {
    return new rocks_flushoptions_t;
  }

  void rocks_flushoptions_destroy(rocks_flushoptions_t* opt) {
    delete opt;
  }

  void rocks_flushoptions_set_wait(rocks_flushoptions_t* opt, unsigned char v) {
    opt->rep.wait = v;
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
