#pragma once

#ifdef __cplusplus
extern "C" {
#endif
#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>

/* slice is the same in rocksdb & rust */
typedef struct Slice {
  const char* data_;
  size_t size_;
} Slice;
typedef struct rocks_pinnable_slice_t rocks_pinnable_slice_t;

/* db.h */
typedef struct rocks_column_family_descriptor_t rocks_column_family_descriptor_t;
typedef struct rocks_column_family_handle_t rocks_column_family_handle_t;
typedef struct rocks_db_t rocks_db_t;

/* options.h */
typedef struct rocks_cfoptions_t rocks_cfoptions_t;
typedef struct rocks_dbpath_t rocks_dbpath_t;
typedef struct rocks_dboptions_t rocks_dboptions_t;
typedef struct rocks_options_t rocks_options_t;
typedef struct rocks_readoptions_t rocks_readoptions_t;
typedef struct rocks_writeoptions_t rocks_writeoptions_t;
typedef struct rocks_flushoptions_t rocks_flushoptions_t;
typedef struct rocks_fifo_compaction_options_t rocks_fifo_compaction_options_t;
typedef struct rocks_compaction_options_t rocks_compaction_options_t;
typedef struct rocks_compactrange_options_t rocks_compactrange_options_t;
typedef struct rocks_ingestexternalfile_options_t rocks_ingestexternalfile_options_t;

/* status.h */
typedef struct rocks_status_t rocks_status_t;

/* rate_limiter.h */
typedef struct rocks_ratelimiter_t rocks_ratelimiter_t;

/* env */
typedef struct rocks_envoptions_t rocks_envoptions_t;
typedef struct rocks_logger_t rocks_logger_t;
typedef struct rocks_env_t rocks_env_t;

/* snapshot*/
typedef struct rocks_snapshot_t rocks_snapshot_t;

/* iterator */
typedef struct rocks_iterator_t rocks_iterator_t;

/* write_batch */
typedef struct rocks_writebatch_t rocks_writebatch_t;
typedef struct rocks_raw_writebatch_t rocks_raw_writebatch_t;
typedef struct rocks_writebatch_handler_t rocks_writebatch_handler_t;

/* table */
typedef struct rocks_block_based_table_options_t rocks_block_based_table_options_t;
typedef struct rocks_cuckoo_table_options_t rocks_cuckoo_table_options_t;
typedef struct rocks_plain_table_options_t rocks_plain_table_options_t;

/* filter_policy */
typedef struct rocks_raw_filterpolicy_t rocks_raw_filterpolicy_t;

/* cache */
typedef struct rocks_cache_t rocks_cache_t;

/* persistent_cache */
typedef struct rocks_persistent_cache_t rocks_persistent_cache_t;

/* merge_operator.h */
typedef struct rocks_associative_mergeoperator_t rocks_associative_mergeoperator_t;
typedef struct rocks_mergeoperator_t rocks_mergeoperator_t;

/* comparator.h */
typedef struct rocks_comparator_t rocks_comparator_t;     /* for rust trait object */
typedef struct rocks_c_comparator_t rocks_c_comparator_t; /* for c */

/* sst_file_writer.h */
typedef struct rocks_sst_file_writer_t rocks_sst_file_writer_t;
typedef struct rocks_external_sst_file_info_t rocks_external_sst_file_info_t;

/* db_dump_tool */
typedef struct rocks_dump_options_t rocks_dump_options_t;
typedef struct rocks_undump_options_t rocks_undump_options_t;

/* iostats_context */
typedef struct rocks_iostats_context_t rocks_iostats_context_t;

/* perf_context */
typedef struct rocks_perf_context_t rocks_perf_context_t;

/* statistics */
typedef struct rocks_statistics_t rocks_statistics_t;
typedef struct rocks_histogram_data_t rocks_histogram_data_t;

/* metadata */
typedef struct rocks_livefiles_t rocks_livefiles_t;
typedef struct rocks_column_family_metadata_t rocks_column_family_metadata_t;

/* universal_compaction */
typedef struct rocks_universal_compaction_options_t rocks_universal_compaction_options_t;

/* transaction_log */
typedef struct rocks_logfiles_t rocks_logfiles_t;
typedef struct rocks_transaction_log_iterator_t rocks_transaction_log_iterator_t;

/* table_properties */
typedef struct rocks_table_props_collection_t rocks_table_props_collection_t;

typedef struct rocks_table_props_collection_iter_t rocks_table_props_collection_iter_t;

typedef struct rocks_table_props_t rocks_table_props_t;

typedef struct rocks_user_collected_props_t rocks_user_collected_props_t;

typedef struct rocks_user_collected_props_iter_t rocks_user_collected_props_iter_t;

typedef struct rocks_table_props_collector_t rocks_table_props_collector_t;

typedef struct rocks_table_props_collector_factory_t rocks_table_props_collector_factory_t;

/* write_buffer_manager */
typedef struct rocks_write_buffer_manager_t rocks_write_buffer_manager_t;

/* debug */

typedef struct rocks_key_version_t rocks_key_version_t;

typedef struct rocks_key_version_collection_t rocks_key_version_collection_t;

/* listener */
typedef struct rocks_event_listener_t rocks_event_listener_t;

typedef struct rocks_flush_job_info_t rocks_flush_job_info_t;
typedef struct rocks_table_file_deletion_info_t rocks_table_file_deletion_info_t;
typedef struct rocks_compaction_job_info_t rocks_compaction_job_info_t;
typedef struct rocks_compaction_job_stats_t rocks_compaction_job_stats_t;
typedef struct rocks_table_file_creation_info_t rocks_table_file_creation_info_t;
typedef struct rocks_table_file_creation_brief_info_t rocks_table_file_creation_brief_info_t;
typedef struct rocks_mem_table_info_t rocks_mem_table_info_t;
typedef struct rocks_external_file_ingestion_info_t rocks_external_file_ingestion_info_t;

/* thread_status */
typedef struct rocks_thread_status_t rocks_thread_status_t;

/* aux */
typedef struct cxx_string_vector_t cxx_string_vector_t;
typedef struct cxx_string_t cxx_string_t; /* std::string */

/* ****************************** functions ****************************** */

/* status */
void rocks_status_destroy(rocks_status_t* s);

int rocks_status_code(rocks_status_t* s);
int rocks_status_subcode(rocks_status_t* s);
int rocks_status_severity(rocks_status_t* s);
const char* rocks_status_get_state(rocks_status_t* s);

/* slice */
rocks_pinnable_slice_t* rocks_pinnable_slice_create();

void rocks_pinnable_slice_destroy(rocks_pinnable_slice_t* s);

const char* rocks_pinnable_slice_data(rocks_pinnable_slice_t* s);

size_t rocks_pinnable_slice_size(rocks_pinnable_slice_t* s);

/* ColumnFamilyDescriptor */
const char* rocks_column_family_descriptor_get_name(const rocks_column_family_descriptor_t* desc);

rocks_cfoptions_t* rocks_column_family_descriptor_get_cfoptions(rocks_column_family_descriptor_t* desc);

/* options.h */
/*    start */
rocks_options_t* rocks_options_create();

void rocks_options_destroy(rocks_options_t* options);

rocks_dboptions_t* rocks_dboptions_create();

void rocks_dboptions_destroy(rocks_dboptions_t* options);

rocks_cfoptions_t* rocks_cfoptions_create();

void rocks_cfoptions_destroy(rocks_cfoptions_t* options);

rocks_options_t* rocks_options_create_from_db_cf_options(rocks_dboptions_t* dbopt, rocks_cfoptions_t* cfopt);

rocks_dboptions_t* rocks_dboptions_create_from_options(rocks_options_t* options);

rocks_cfoptions_t* rocks_cfoptions_create_from_options(rocks_options_t* options);

// cfoptions

void rocks_cfoptions_optimize_for_small_db(rocks_cfoptions_t* opt);

void rocks_cfoptions_optimize_for_point_lookup(rocks_cfoptions_t* opt, uint64_t block_cache_size_mb);

void rocks_cfoptions_optimize_level_style_compaction(rocks_cfoptions_t* opt, uint64_t memtable_memory_budget);

void rocks_cfoptions_optimize_universal_style_compaction(rocks_cfoptions_t* opt, uint64_t memtable_memory_budget);

void rocks_cfoptions_set_merge_operator_by_assoc_op_trait(rocks_cfoptions_t* opt, void* op_trait_obj);

void rocks_cfoptions_set_merge_operator_by_merge_op_trait(rocks_cfoptions_t* opt, void* op_trait_obj);

void rocks_cfoptions_set_comparator_by_trait(rocks_cfoptions_t* opt, void* cp_trait_obj);

void rocks_cfoptions_set_compaction_filter_by_trait(rocks_cfoptions_t* opt, void* filter_trait_obj);

void rocks_cfoptions_set_bitwise_comparator(rocks_cfoptions_t* opt, unsigned char reversed);

/*
void rocks_cfoptions_set_compaction_filter(
                                       rocks_options_t* opt,
                                       rocks_compactionfilter_t* filter);

void rocks_cfoptions_set_compaction_filter_factory(rocks_options_t* opt,
rocks_compactionfilterfactory_t* factory);
*/

void rocks_cfoptions_set_write_buffer_size(rocks_cfoptions_t* opt, size_t s);

void rocks_cfoptions_set_compression(rocks_cfoptions_t* opt, int t);

void rocks_cfoptions_set_bottommost_compression(rocks_cfoptions_t* opt, int t);

void rocks_cfoptions_set_compression_options(rocks_cfoptions_t* opt, int w_bits, int level, int strategy,
                                             uint32_t max_dict_bytes);

void rocks_cfoptions_set_level0_file_num_compaction_trigger(rocks_cfoptions_t* opt, int n);

void rocks_cfoptions_set_prefix_extractor_by_trait(rocks_cfoptions_t* opt, void* trans_trait_obj);
void rocks_cfoptions_set_prefix_extractor_fixed_prefix(rocks_cfoptions_t* opt, size_t prefix_len);
void rocks_cfoptions_set_prefix_extractor_capped_prefix(rocks_cfoptions_t* opt, size_t cap_len);
void rocks_cfoptions_set_prefix_extractor_noop(rocks_cfoptions_t* opt);

void rocks_cfoptions_set_max_bytes_for_level_base(rocks_cfoptions_t* opt, uint64_t n);

void rocks_cfoptions_set_disable_auto_compactions(rocks_cfoptions_t* opt, unsigned char disable);

// table_factory
void rocks_cfoptions_set_block_based_table_factory(rocks_cfoptions_t* opt,
                                                   rocks_block_based_table_options_t* table_options);
void rocks_cfoptions_set_cuckoo_table_factory(rocks_cfoptions_t* opt, rocks_cuckoo_table_options_t* table_options);
void rocks_cfoptions_set_plain_table_factory(rocks_cfoptions_t* opt, rocks_plain_table_options_t* table_options);

// via AdvancedColumnFamilyOptions

void rocks_cfoptions_set_max_write_buffer_number(rocks_cfoptions_t* opt, int n);

void rocks_cfoptions_set_min_write_buffer_number_to_merge(rocks_cfoptions_t* opt, int n);

void rocks_cfoptions_set_max_write_buffer_number_to_maintain(rocks_cfoptions_t* opt, int n);

void rocks_cfoptions_set_inplace_update_support(rocks_cfoptions_t* opt, unsigned char v);

void rocks_cfoptions_set_inplace_update_num_locks(rocks_cfoptions_t* opt, size_t v);

// inplace_callback

void rocks_cfoptions_set_memtable_prefix_bloom_size_ratio(rocks_cfoptions_t* opt, double v);

void rocks_cfoptions_set_memtable_huge_page_size(rocks_cfoptions_t* opt, size_t v);

void rocks_cfoptions_set_memtable_insert_with_hint_prefix_extractor_by_trait(rocks_cfoptions_t* opt,
                                                                             void* trans_trait_obj);
void rocks_cfoptions_set_memtable_insert_with_hint_prefix_extractor_fixed_prefix(rocks_cfoptions_t* opt,
                                                                                 size_t prefix_len);
void rocks_cfoptions_set_memtable_insert_with_hint_prefix_extractor_capped_prefix(rocks_cfoptions_t* opt,
                                                                                  size_t cap_len);
void rocks_cfoptions_set_memtable_insert_with_hint_prefix_extractor_noop(rocks_cfoptions_t* opt);

void rocks_cfoptions_set_bloom_locality(rocks_cfoptions_t* opt, uint32_t v);

void rocks_cfoptions_set_arena_block_size(rocks_cfoptions_t* opt, size_t v);

void rocks_cfoptions_set_compression_per_level(rocks_cfoptions_t* opt, const int* level_values, size_t num_levels);

void rocks_cfoptions_set_num_levels(rocks_cfoptions_t* opt, int n);

void rocks_cfoptions_set_level0_slowdown_writes_trigger(rocks_cfoptions_t* opt, int n);

void rocks_cfoptions_set_level0_stop_writes_trigger(rocks_cfoptions_t* opt, int n);

void rocks_cfoptions_set_target_file_size_base(rocks_cfoptions_t* opt, uint64_t n);

void rocks_cfoptions_set_target_file_size_multiplier(rocks_cfoptions_t* opt, int n);

void rocks_cfoptions_set_level_compaction_dynamic_level_bytes(rocks_cfoptions_t* opt, unsigned char v);

void rocks_cfoptions_set_max_bytes_for_level_multiplier(rocks_cfoptions_t* opt, double n);

void rocks_cfoptions_set_max_bytes_for_level_multiplier_additional(rocks_cfoptions_t* opt, const int* level_values,
                                                                   size_t num_levels);

void rocks_cfoptions_set_max_compaction_bytes(rocks_cfoptions_t* opt, uint64_t n);

void rocks_cfoptions_set_soft_pending_compaction_bytes_limit(rocks_cfoptions_t* opt, uint64_t v);

void rocks_cfoptions_set_hard_pending_compaction_bytes_limit(rocks_cfoptions_t* opt, uint64_t v);

void rocks_cfoptions_set_compaction_style(rocks_cfoptions_t* opt, int style);

void rocks_cfoptions_set_compaction_pri(rocks_cfoptions_t* opt, int pri);

void rocks_cfoptions_set_universal_compaction_options(rocks_cfoptions_t* opt,
                                                      rocks_universal_compaction_options_t* uco);

void rocks_cfoptions_set_fifo_compaction_options(rocks_cfoptions_t* opt, rocks_fifo_compaction_options_t* fifo);

void rocks_cfoptions_set_max_sequential_skip_in_iterations(rocks_cfoptions_t* opt, uint64_t v);

// memtable_factory
void rocks_cfoptions_set_memtable_vector_rep(rocks_cfoptions_t* opt, size_t count);

void rocks_cfoptions_set_hash_skip_list_rep(rocks_cfoptions_t* opt, size_t bucket_count, int32_t skiplist_height,
                                            int32_t skiplist_branching_factor);

void rocks_cfoptions_set_hash_link_list_rep(rocks_cfoptions_t* opt, size_t bucket_count);

void rocks_cfoptions_set_hash_cuckoo_rep(rocks_cfoptions_t* opt, size_t write_buffer_size, size_t average_data_size,
                                         unsigned int hash_function_count);

void rocks_cfoptions_add_table_properties_collector_factories_by_trait(rocks_cfoptions_t* opt, void* factory_trait_obj);

void rocks_cfoptions_set_max_successive_merges(rocks_cfoptions_t* opt, size_t v);

void rocks_cfoptions_set_optimize_filters_for_hits(rocks_cfoptions_t* opt, unsigned char v);

void rocks_cfoptions_set_paranoid_file_checks(rocks_cfoptions_t* opt, unsigned char v);

void rocks_cfoptions_set_force_consistency_checks(rocks_cfoptions_t* opt, unsigned char v);

void rocks_cfoptions_set_report_bg_io_stats(rocks_cfoptions_t* opt, unsigned char v);

// dboptions

void rocks_dboptions_optimize_for_small_db(rocks_dboptions_t* opt);

void rocks_dboptions_increase_parallelism(rocks_dboptions_t* opt, int total_threads);

void rocks_dboptions_set_create_if_missing(rocks_dboptions_t* opt, unsigned char v);

void rocks_dboptions_set_create_missing_column_families(rocks_dboptions_t* opt, unsigned char v);

void rocks_dboptions_set_error_if_exists(rocks_dboptions_t* opt, unsigned char v);

void rocks_dboptions_set_paranoid_checks(rocks_dboptions_t* opt, unsigned char v);

void rocks_dboptions_set_env(rocks_dboptions_t* opt, rocks_env_t* env);

void rocks_dboptions_set_ratelimiter(rocks_dboptions_t* opt, rocks_ratelimiter_t* limiter);

// void rocks_dboptions_set_sst_file_manager(rocks_dboptions_t* opt,
// rocks_sst_file_manager_t* manager);

void rocks_dboptions_set_info_log(rocks_dboptions_t* opt, rocks_logger_t* l);

void rocks_dboptions_set_info_log_level(rocks_dboptions_t* opt, int v);

void rocks_dboptions_set_max_open_files(rocks_dboptions_t* opt, int n);

void rocks_dboptions_set_max_file_opening_threads(rocks_dboptions_t* opt, int n);

void rocks_dboptions_set_max_total_wal_size(rocks_dboptions_t* opt, uint64_t n);

void rocks_dboptions_set_statistics(rocks_dboptions_t* opt, rocks_statistics_t* stat);

void rocks_dboptions_set_use_fsync(rocks_dboptions_t* opt, unsigned char use_fsync);

void rocks_dboptions_set_db_paths(rocks_dboptions_t* opt, const char* const* paths, const size_t* path_lens,
                                  const uint64_t* target_sizes, int size);

void rocks_dboptions_set_db_log_dir(rocks_dboptions_t* opt, const char* db_log_dir, size_t len);

void rocks_dboptions_set_wal_dir(rocks_dboptions_t* opt, const char* v, size_t len);

void rocks_dboptions_set_delete_obsolete_files_period_micros(rocks_dboptions_t* opt, uint64_t v);

void rocks_dboptions_set_max_background_jobs(rocks_dboptions_t* opt, int n);

void rocks_dboptions_set_max_subcompactions(rocks_dboptions_t* opt, uint32_t n);

void rocks_dboptions_set_max_log_file_size(rocks_dboptions_t* opt, size_t v);

void rocks_dboptions_set_log_file_time_to_roll(rocks_dboptions_t* opt, size_t v);

void rocks_dboptions_set_keep_log_file_num(rocks_dboptions_t* opt, size_t v);

void rocks_dboptions_set_recycle_log_file_num(rocks_dboptions_t* opt, size_t v);

void rocks_dboptions_set_max_manifest_file_size(rocks_dboptions_t* opt, uint64_t v);

void rocks_dboptions_set_table_cache_numshardbits(rocks_dboptions_t* opt, int v);

void rocks_dboptions_set_wal_ttl_seconds(rocks_dboptions_t* opt, uint64_t ttl);

void rocks_dboptions_set_wal_size_limit_mb(rocks_dboptions_t* opt, uint64_t limit);

void rocks_dboptions_set_manifest_preallocation_size(rocks_dboptions_t* opt, size_t v);

void rocks_dboptions_set_allow_mmap_reads(rocks_dboptions_t* opt, unsigned char v);

void rocks_dboptions_set_allow_mmap_writes(rocks_dboptions_t* opt, unsigned char v);

void rocks_dboptions_set_use_direct_reads(rocks_dboptions_t* opt, unsigned char v);

void rocks_dboptions_set_use_direct_io_for_flush_and_compaction(rocks_dboptions_t* opt, unsigned char v);

void rocks_dboptions_set_allow_fallocate(rocks_dboptions_t* opt, unsigned char v);

void rocks_dboptions_set_is_fd_close_on_exec(rocks_dboptions_t* opt, unsigned char v);

void rocks_dboptions_set_stats_dump_period_sec(rocks_dboptions_t* opt, unsigned int v);

void rocks_dboptions_set_advise_random_on_open(rocks_dboptions_t* opt, unsigned char v);

void rocks_dboptions_set_db_write_buffer_size(rocks_dboptions_t* opt, size_t s);

void rocks_dboptions_set_write_buffer_manager(rocks_dboptions_t* opt, rocks_write_buffer_manager_t* manager);

void rocks_dboptions_set_access_hint_on_compaction_start(rocks_dboptions_t* opt, int v);

void rocks_dboptions_set_new_table_reader_for_compaction_inputs(rocks_dboptions_t* opt, unsigned char v);
void rocks_dboptions_set_compaction_readahead_size(rocks_dboptions_t* opt, size_t s);
void rocks_dboptions_set_random_access_max_buffer_size(rocks_dboptions_t* opt, size_t s);
void rocks_dboptions_set_writable_file_max_buffer_size(rocks_dboptions_t* opt, size_t s);
void rocks_dboptions_set_use_adaptive_mutex(rocks_dboptions_t* opt, unsigned char v);
void rocks_dboptions_set_bytes_per_sync(rocks_dboptions_t* opt, uint64_t v);
void rocks_dboptions_set_wal_bytes_per_sync(rocks_dboptions_t* opt, uint64_t v);

void rocks_dboptions_add_listener(rocks_dboptions_t* opt, void* listener_trait_obj);

void rocks_dboptions_set_enable_thread_tracking(rocks_dboptions_t* opt, unsigned char v);

void rocks_dboptions_set_delayed_write_rate(rocks_dboptions_t* opt, uint64_t v);

void rocks_dboptions_set_allow_concurrent_memtable_write(rocks_dboptions_t* opt, unsigned char v);

void rocks_dboptions_set_enable_write_thread_adaptive_yield(rocks_dboptions_t* opt, unsigned char v);

void rocks_dboptions_set_write_thread_max_yield_usec(rocks_dboptions_t* opt, uint64_t v);

void rocks_dboptions_set_write_thread_slow_yield_usec(rocks_dboptions_t* opt, uint64_t v);

void rocks_dboptions_set_skip_stats_update_on_db_open(rocks_dboptions_t* opt, unsigned char v);

void rocks_dboptions_set_wal_recovery_mode(rocks_dboptions_t* opt, int mode);

void rocks_dboptions_set_allow_2pc(rocks_dboptions_t* opt, unsigned char v);

// FIXME: mem leaks?
void rocks_dboptions_set_row_cache(rocks_dboptions_t* opt, rocks_cache_t* cache);

/*
void rocks_dboptions_set_wal_filter(rocks_dboptions_t* opt, rocks_wal_filter_t*
filter);
*/

void rocks_dboptions_set_fail_if_options_file_error(rocks_dboptions_t* opt, unsigned char v);

void rocks_dboptions_set_dump_malloc_stats(rocks_dboptions_t* opt, unsigned char v);

void rocks_dboptions_set_avoid_flush_during_recovery(rocks_dboptions_t* opt, unsigned char v);

void rocks_dboptions_set_avoid_flush_during_shutdown(rocks_dboptions_t* opt, unsigned char v);

void rocks_dboptions_set_allow_ingest_behind(rocks_dboptions_t* opt, unsigned char v);

void rocks_dboptions_set_concurrent_prepare(rocks_dboptions_t* opt, unsigned char v);

void rocks_dboptions_set_manual_wal_flush(rocks_dboptions_t* opt, unsigned char v);

// opt

void rocks_options_prepare_for_bulk_load(rocks_options_t* opt);

void rocks_options_optimize_for_small_db(rocks_options_t* opt);

/*
  char *rocks_options_statistics_get_string(rocks_options_t *opt) {
  rocksdb::Statistics *statistics = opt->rep.statistics.get();
  if (statistics) {
  return strdup(statistics->ToString().c_str());
  }
  return nullptr;
  }
  */

/*    end */

/* readoptions */
rocks_readoptions_t* rocks_readoptions_create();

rocks_readoptions_t* rocks_readoptions_new(unsigned char cksum, unsigned char cache);

void rocks_readoptions_destroy(rocks_readoptions_t* opt);

void rocks_readoptions_set_verify_checksums(rocks_readoptions_t* opt, unsigned char v);

void rocks_readoptions_set_fill_cache(rocks_readoptions_t* opt, unsigned char v);

void rocks_readoptions_set_snapshot(rocks_readoptions_t* opt, const rocks_snapshot_t* snap);

void rocks_readoptions_set_iterate_lower_bound(rocks_readoptions_t* opt, const char* key, size_t keylen);

void rocks_readoptions_set_iterate_upper_bound(rocks_readoptions_t* opt, const char* key, size_t keylen);

void rocks_readoptions_set_read_tier(rocks_readoptions_t* opt, int v);

void rocks_readoptions_set_tailing(rocks_readoptions_t* opt, unsigned char v);

void rocks_readoptions_set_managed(rocks_readoptions_t* opt, unsigned char v);

void rocks_readoptions_set_readahead_size(rocks_readoptions_t* opt, size_t v);

void rocks_readoptions_set_max_skippable_internal_keys(rocks_readoptions_t* opt, uint64_t v);

void rocks_readoptions_set_pin_data(rocks_readoptions_t* opt, unsigned char v);

void rocks_readoptions_set_total_order_seek(rocks_readoptions_t* opt, unsigned char v);

void rocks_readoptions_set_prefix_same_as_start(rocks_readoptions_t* opt, unsigned char v);

void rocks_readoptions_set_ignore_range_deletions(rocks_readoptions_t* opt, unsigned char v);

void rocks_readoptions_set_background_purge_on_iterator_cleanup(rocks_readoptions_t* opt, unsigned char v);

/* > writeoptions */
rocks_writeoptions_t* rocks_writeoptions_create();

void rocks_writeoptions_destroy(rocks_writeoptions_t* opt);

void rocks_writeoptions_set_sync(rocks_writeoptions_t* opt, unsigned char v);

void rocks_writeoptions_set_disable_wal(rocks_writeoptions_t* opt, unsigned char v);

void rocks_writeoptions_set_ignore_missing_column_families(rocks_writeoptions_t* opt, unsigned char v);

void rocks_writeoptions_set_no_slowdown(rocks_writeoptions_t* opt, unsigned char v);

void rocks_writeoptions_set_low_pri(rocks_writeoptions_t* opt, unsigned char v);

/* > compactrange_options */
rocks_compactrange_options_t* rocks_compactrange_options_create();

void rocks_compactrange_options_destroy(rocks_compactrange_options_t* opt);

void rocks_compactrange_options_set_exclusive_manual_compaction(rocks_compactrange_options_t* opt, unsigned char v);

void rocks_compactrange_options_set_change_level(rocks_compactrange_options_t* opt, unsigned char v);

void rocks_compactrange_options_set_target_level(rocks_compactrange_options_t* opt, int32_t v);

void rocks_compactrange_options_set_target_path_id(rocks_compactrange_options_t* opt, uint32_t v);

void rocks_compactrange_options_set_bottommost_level_compaction(rocks_compactrange_options_t* opt, int v);

/* > ingestexternalfile_options */
rocks_ingestexternalfile_options_t* rocks_ingestexternalfile_options_create();

void rocks_ingestexternalfile_options_destroy(rocks_ingestexternalfile_options_t* opt);

void rocks_ingestexternalfile_options_set_move_files(rocks_ingestexternalfile_options_t* opt, unsigned char v);
void rocks_ingestexternalfile_options_set_snapshot_consistency(rocks_ingestexternalfile_options_t* opt,
                                                               unsigned char v);
void rocks_ingestexternalfile_options_set_allow_global_seqno(rocks_ingestexternalfile_options_t* opt, unsigned char v);
void rocks_ingestexternalfile_options_set_allow_blocking_flush(rocks_ingestexternalfile_options_t* opt,
                                                               unsigned char v);
void rocks_ingestexternalfile_options_set_ingest_behind(rocks_ingestexternalfile_options_t* opt, unsigned char v);

/* > flushoptions */
rocks_flushoptions_t* rocks_flushoptions_create();
void rocks_flushoptions_destroy(rocks_flushoptions_t* options);

void rocks_flushoptions_set_wait(rocks_flushoptions_t* options, unsigned char v);

/* > misc */
rocks_logger_t* rocks_create_logger_from_options(const char* path, rocks_options_t* opts, rocks_status_t** status);

/* > fifo_compaction_options */
rocks_fifo_compaction_options_t* rocks_fifo_compaction_options_create();

void rocks_fifo_compaction_options_set_max_table_files_size(rocks_fifo_compaction_options_t* fifo_opts, uint64_t size);

void rocks_fifo_compaction_options_set_ttl(rocks_fifo_compaction_options_t* fifo_opts, uint64_t val);

void rocks_fifo_compaction_options_set_allow_compaction(rocks_fifo_compaction_options_t* fifo_opts, unsigned char val);

void rocks_fifo_compaction_options_destroy(rocks_fifo_compaction_options_t* fifo_opts);

/* > compaction_options */
rocks_compaction_options_t* rocks_compaction_options_create();
void rocks_compaction_options_destroy(rocks_compaction_options_t* opts);

void rocks_compaction_options_set_compression(rocks_compaction_options_t* opts, int val);
void rocks_compaction_options_set_output_file_size_limit(rocks_compaction_options_t* opts, uint64_t val);

/* db.h */

/* > rocks_column_family_handle_t */
const char* rocks_column_family_handle_get_name(const rocks_column_family_handle_t* handle);

uint32_t rocks_column_family_handle_get_id(const rocks_column_family_handle_t* handle);

/* > rocks_db_t */
rocks_db_t* rocks_db_open(const rocks_options_t* options, const char* name, rocks_status_t** status);

void rocks_db_close(rocks_db_t* db);

rocks_db_t* rocks_db_open_for_read_only(const rocks_options_t* options, const char* name,
                                        unsigned char error_if_log_file_exist, rocks_status_t** status);

rocks_db_t* rocks_db_open_as_secondary(const rocks_options_t* options, const char* name, const char* secondary_path,
                                       rocks_status_t** status);

rocks_db_t* rocks_db_open_as_secondary_column_families(const rocks_options_t* options, const char* name,
                                                       const char* secondary_path, int num_column_families,
                                                       const char* const* column_family_names,
                                                       const rocks_cfoptions_t* const* column_family_options,
                                                       rocks_column_family_handle_t** column_family_handles,
                                                       rocks_status_t** status);

void rocks_db_try_catch_up_with_primary(rocks_db_t* db, rocks_status_t** status);

rocks_db_t* rocks_db_open_column_families(const rocks_dboptions_t* db_options, const char* name,
                                          int num_column_families, const char* const* column_family_names,
                                          const rocks_cfoptions_t* const* column_family_options,
                                          rocks_column_family_handle_t** column_family_handles,
                                          rocks_status_t** status);

rocks_db_t* rocks_db_open_for_read_only_column_families(const rocks_dboptions_t* db_options, const char* name,
                                                        int num_column_families, const char* const* column_family_names,
                                                        const rocks_cfoptions_t* const* column_family_options,
                                                        rocks_column_family_handle_t** column_family_handles,
                                                        unsigned char error_if_log_file_exist, rocks_status_t** status);

char** rocks_db_list_column_families(const rocks_options_t* options, const char* name, size_t* lencfs,
                                     rocks_status_t** status);

void rocks_db_list_column_families_destroy(char** list, size_t len);

rocks_column_family_handle_t* rocks_db_create_column_family(rocks_db_t* db,
                                                            const rocks_cfoptions_t* column_family_options,
                                                            const char* column_family_name, rocks_status_t** status);

rocks_column_family_handle_t* rocks_db_default_column_family(rocks_db_t* db);

void rocks_db_drop_column_family(rocks_db_t* db, rocks_column_family_handle_t* handle, rocks_status_t** status);

/* FIXME: when to use? */
void rocks_db_destroy_column_family_handle(rocks_db_t* db, rocks_column_family_handle_t* handle,
                                           rocks_status_t** status);

void rocks_column_family_handle_destroy(rocks_column_family_handle_t* handle);

void rocks_db_put(rocks_db_t* db, const rocks_writeoptions_t* options, const char* key, size_t keylen, const char* val,
                  size_t vallen, rocks_status_t** status);

/*
void rocks_db_put_slice(
                      rocks_db_t* db,
                      const rocks_writeoptions_t* options,
                      const Slice* key, const Slice* value,
                      rocks_status_t** status);
*/
void rocks_db_put_cf(rocks_db_t* db, const rocks_writeoptions_t* options, rocks_column_family_handle_t* column_family,
                     const char* key, size_t keylen, const char* val, size_t vallen, rocks_status_t** status);

void rocks_db_delete(rocks_db_t* db, const rocks_writeoptions_t* options, const char* key, size_t keylen,
                     rocks_status_t** status);

void rocks_db_delete_cf(rocks_db_t* db, const rocks_writeoptions_t* options,
                        rocks_column_family_handle_t* column_family, const char* key, size_t keylen,
                        rocks_status_t** status);

void rocks_db_single_delete(rocks_db_t* db, const rocks_writeoptions_t* options, const char* key, size_t keylen,
                            rocks_status_t** status);

void rocks_db_single_delete_cf(rocks_db_t* db, const rocks_writeoptions_t* options,
                               rocks_column_family_handle_t* column_family, const char* key, size_t keylen,
                               rocks_status_t** status);

void rocks_db_delete_range_cf(rocks_db_t* db, const rocks_writeoptions_t* options,
                              rocks_column_family_handle_t* column_family, const char* begin_key, size_t begin_keylen,
                              const char* end_key, size_t end_keylen, rocks_status_t** status);

void rocks_db_merge(rocks_db_t* db, const rocks_writeoptions_t* options, const char* key, size_t keylen,
                    const char* val, size_t vallen, rocks_status_t** status);

void rocks_db_merge_cf(rocks_db_t* db, const rocks_writeoptions_t* options, rocks_column_family_handle_t* column_family,
                       const char* key, size_t keylen, const char* val, size_t vallen, rocks_status_t** status);

void rocks_db_write(rocks_db_t* db, const rocks_writeoptions_t* options, rocks_raw_writebatch_t* batch,
                    rocks_status_t** status);

void rocks_db_get_pinnable(rocks_db_t* db, const rocks_readoptions_t* options, const char* key, size_t keylen,
                           rocks_pinnable_slice_t* value, rocks_status_t** status);

void rocks_db_get_cf_pinnable(rocks_db_t* db, const rocks_readoptions_t* options,
                              rocks_column_family_handle_t* column_family, const char* key, size_t keylen,
                              rocks_pinnable_slice_t* value, rocks_status_t** status);

void rocks_db_multi_get(rocks_db_t* db, const rocks_readoptions_t* options, size_t num_keys,
                        const char* const* keys_list, const size_t* keys_list_sizes, char** values_list,
                        size_t* values_list_sizes, rocks_status_t** status);

void rocks_db_multi_get_cf(rocks_db_t* db, const rocks_readoptions_t* options,
                           const rocks_column_family_handle_t* const* column_families, size_t num_keys,
                           const char* const* keys_list, const size_t* keys_list_sizes, char** values_list,
                           size_t* values_list_sizes, rocks_status_t** status);

unsigned char rocks_db_key_may_exist(rocks_db_t* db, const rocks_readoptions_t* options, const char* key,
                                     size_t key_len, void* value, unsigned char* value_found);

unsigned char rocks_db_key_may_exist_cf(rocks_db_t* db, const rocks_readoptions_t* options,
                                        const rocks_column_family_handle_t* column_family, const char* key,
                                        size_t key_len, void* value, unsigned char* value_found);

rocks_iterator_t* rocks_db_create_iterator(rocks_db_t* db, const rocks_readoptions_t* options);

rocks_iterator_t* rocks_db_create_iterator_cf(rocks_db_t* db, const rocks_readoptions_t* options,
                                              rocks_column_family_handle_t* column_family);

void rocks_db_create_iterators(rocks_db_t* db, rocks_readoptions_t* opts,
                               rocks_column_family_handle_t** column_families, rocks_iterator_t** iterators,
                               size_t size, rocks_status_t** status);

rocks_snapshot_t* rocks_db_get_snapshot(rocks_db_t* db);

void rocks_db_release_snapshot(rocks_db_t* db, rocks_snapshot_t* snapshot);

unsigned char rocks_db_get_property(rocks_db_t* db, const char* prop, const size_t prop_len,
                                    void* value); /* *mut String */

unsigned char rocks_db_get_property_cf(rocks_db_t* db, rocks_column_family_handle_t* cf, const char* prop,
                                       const size_t prop_len, void* value);

unsigned char rocks_db_get_int_property(rocks_db_t* db, const char* prop, const size_t prop_len, uint64_t* value);

unsigned char rocks_db_get_int_property_cf(rocks_db_t* db, rocks_column_family_handle_t* cf, const char* prop,
                                           const size_t prop_len, uint64_t* value);

unsigned char rocks_db_get_aggregated_int_property(rocks_db_t* db, const char* prop, const size_t prop_len,
                                                   uint64_t* value);

void rocks_db_compact_range(rocks_db_t* db, const char* start_key, size_t start_key_len, const char* limit_key,
                            size_t limit_key_len);

void rocks_db_compact_range_cf(rocks_db_t* db, rocks_column_family_handle_t* column_family, const char* start_key,
                               size_t start_key_len, const char* limit_key, size_t limit_key_len);

void rocks_db_compact_range_opt(rocks_db_t* db, rocks_compactrange_options_t* opt, const char* start_key,
                                size_t start_key_len, const char* limit_key, size_t limit_key_len,
                                rocks_status_t** status);

void rocks_db_compact_range_opt_cf(rocks_db_t* db, rocks_compactrange_options_t* opt,
                                   rocks_column_family_handle_t* column_family, const char* start_key,
                                   size_t start_key_len, const char* limit_key, size_t limit_key_len,
                                   rocks_status_t** status);

void rocks_db_set_options_cf(rocks_db_t* db, rocks_column_family_handle_t* column_family, size_t num_options,
                             const char* const* keys, const size_t* key_lens, const char* const* vals,
                             const size_t* val_lens, rocks_status_t** status);

void rocks_db_set_db_options(rocks_db_t* db, size_t num_options, const char* const* keys, const size_t* key_lens,
                             const char* const* vals, const size_t* val_lens, rocks_status_t** status);

void rocks_db_compact_files(rocks_db_t* db, rocks_compaction_options_t* opt, size_t num_files,
                            const char* const* file_names, const size_t* file_name_lens, const int output_level,
                            const int output_path_id, rocks_status_t** status);

void rocks_db_pause_background_work(rocks_db_t* db, rocks_status_t** status);
void rocks_db_continue_background_work(rocks_db_t* db, rocks_status_t** status);

void rocks_db_enable_auto_compaction(rocks_db_t* db, const rocks_column_family_handle_t* const* column_families,
                                     size_t cf_len, rocks_status_t** status);

int rocks_db_number_levels_cf(rocks_db_t* db, rocks_column_family_handle_t* column_family);
int rocks_db_number_levels(rocks_db_t* db);

int rocks_db_max_mem_compaction_level_cf(rocks_db_t* db, rocks_column_family_handle_t* column_family);
int rocks_db_max_mem_compaction_level(rocks_db_t* db);

int rocks_db_level0_stop_write_trigger_cf(rocks_db_t* db, rocks_column_family_handle_t* column_family);
int rocks_db_level0_stop_write_trigger(rocks_db_t* db);

void rocks_db_get_approximate_sizes_cf(rocks_db_t* db, rocks_column_family_handle_t* column_family, size_t num_ranges,
                                       const char* const* range_start_ptrs, const size_t* range_start_lens,
                                       const char* const* range_limit_ptrs, const size_t* range_limit_lens,
                                       uint64_t* sizes);

void rocks_db_get_approximate_memtable_stats_cf(rocks_db_t* db, rocks_column_family_handle_t* column_family,
                                                const char* range_start_ptr, size_t range_start_len,
                                                const char* range_limit_ptr, size_t range_limit_len, uint64_t* count,
                                                uint64_t* size);

void rocks_db_get_name(rocks_db_t* db, void* s);

void rocks_db_flush(rocks_db_t* db, rocks_flushoptions_t* options, rocks_status_t** status);
void rocks_db_flush_cf(rocks_db_t* db, rocks_flushoptions_t* options, rocks_column_family_handle_t* column_family,
                       rocks_status_t** status);

void rocks_db_sync_wal(rocks_db_t* db, rocks_status_t** status);

uint64_t rocks_db_get_latest_sequence_number(rocks_db_t* db);

void rocks_db_disable_file_deletions(rocks_db_t* db, rocks_status_t** status);

void rocks_db_enable_file_deletions(rocks_db_t* db, unsigned char force, rocks_status_t** status);

cxx_string_vector_t* rocks_db_get_live_files(rocks_db_t* db, unsigned char flush_memtable, uint64_t* manifest_file_size,
                                             rocks_status_t** status);

rocks_logfiles_t* rocks_db_get_sorted_wal_files(rocks_db_t* db, rocks_status_t** status);

rocks_transaction_log_iterator_t* rocks_db_get_update_since(rocks_db_t* db, uint64_t seq_no, rocks_status_t** status);

void rocks_db_delete_file(rocks_db_t* db, const char* name, size_t name_len, rocks_status_t** status);

const rocks_livefiles_t* rocks_db_get_livefiles_metadata(rocks_db_t* db);

const rocks_column_family_metadata_t* rocks_db_get_column_family_metadata(rocks_db_t* db,
                                                                          rocks_column_family_handle_t* column_family);

void rocks_db_ingest_external_file(rocks_db_t* db, const char* const* file_list, const size_t* file_list_sizes,
                                   size_t file_len, const rocks_ingestexternalfile_options_t* options,
                                   rocks_status_t** status);

void rocks_db_ingest_external_file_cf(rocks_db_t* db, rocks_column_family_handle_t* column_family,
                                      const char* const* file_list, const size_t* file_list_sizes, size_t file_len,
                                      const rocks_ingestexternalfile_options_t* options, rocks_status_t** status);

void rocks_db_get_db_identity(rocks_db_t* db,
                              void* identity,  // *mut String
                              rocks_status_t** status);

rocks_table_props_collection_t* rocks_db_get_properties_of_all_tables(rocks_db_t* db, rocks_column_family_handle_t* cf,
                                                                      rocks_status_t** status);

rocks_table_props_collection_t* rocks_db_get_properties_of_tables_in_range(
    rocks_db_t* db, rocks_column_family_handle_t* cf, size_t num_ranges, const char* const* start_keys,
    const size_t* start_key_lens, const char* const* limit_keys, const size_t* limit_key_lens, rocks_status_t** status);

/*    pub fn */
void rocks_destroy_db(const rocks_options_t* options, const char* name, size_t len, rocks_status_t** status);
void rocks_repair_db(const rocks_options_t* options, const char* name, size_t len, rocks_status_t** status);

/* > utilities/info_log_finder.h */

// FIXME: missing on static build?
// cxx_string_vector_t* rocks_db_get_info_log_list(rocks_db_t* db, rocks_status_t** status);

/* rate_limiter.h */
rocks_ratelimiter_t* rocks_ratelimiter_create(int64_t rate_bytes_per_sec, int64_t refill_period_us, int32_t fairness);

void rocks_ratelimiter_destroy(rocks_ratelimiter_t* limiter);

/* env.h */
rocks_env_t* rocks_create_default_env();

rocks_env_t* rocks_create_mem_env();
rocks_env_t* rocks_create_timed_env();

void rocks_env_destroy(rocks_env_t* env);

void rocks_env_set_background_threads(rocks_env_t* env, int n);

void rocks_env_set_high_priority_background_threads(rocks_env_t* env, int n);

void rocks_env_join_all_threads(rocks_env_t* env);

unsigned int rocks_env_get_thread_pool_queue_len(rocks_env_t* env, int pri);

rocks_logger_t* rocks_env_new_logger(rocks_env_t* env, const char* name_ptr, size_t name_len, rocks_status_t** status);

uint64_t rocks_env_now_micros(rocks_env_t* env);

uint64_t rocks_env_now_nanos(rocks_env_t* env);

void rocks_env_sleep_for_microseconds(rocks_env_t* env, int32_t micros);

void rocks_env_get_host_name(rocks_env_t* env, char* name, uint64_t len, rocks_status_t** status);

int64_t rocks_env_get_current_time(rocks_env_t* env, rocks_status_t** status);

cxx_string_t* rocks_env_time_to_string(rocks_env_t* env, uint64_t time);

int rocks_env_get_background_threads(rocks_env_t* env, int pri);

void rocks_env_inc_background_threads_if_needed(rocks_env_t* env, int number, int pri);

void rocks_env_lower_thread_pool_io_priority(rocks_env_t* env, int pool);

rocks_thread_status_t** rocks_env_get_thread_list(rocks_env_t* env, size_t* len);
void rocks_env_get_thread_list_destroy(rocks_thread_status_t** p);

uint64_t rocks_env_get_thread_id(rocks_env_t* env);

rocks_envoptions_t* rocks_envoptions_create();
void rocks_envoptions_destroy(rocks_envoptions_t* opt);

void rocks_envoptions_set_use_mmap_reads(rocks_envoptions_t* opt, unsigned char val);
void rocks_envoptions_set_use_mmap_writes(rocks_envoptions_t* opt, unsigned char val);
void rocks_envoptions_set_use_direct_reads(rocks_envoptions_t* opt, unsigned char val);
void rocks_envoptions_set_use_direct_writes(rocks_envoptions_t* opt, unsigned char val);
void rocks_envoptions_set_allow_fallocate(rocks_envoptions_t* opt, unsigned char val);
void rocks_envoptions_set_fd_cloexec(rocks_envoptions_t* opt, unsigned char val);
void rocks_envoptions_set_bytes_per_sync(rocks_envoptions_t* opt, uint64_t val);
void rocks_envoptions_set_fallocate_with_keep_size(rocks_envoptions_t* opt, unsigned char val);
void rocks_envoptions_set_compaction_readahead_size(rocks_envoptions_t* opt, size_t val);
void rocks_envoptions_set_random_access_max_buffer_size(rocks_envoptions_t* opt, size_t val);
void rocks_envoptions_set_writable_file_max_buffer_size(rocks_envoptions_t* opt, size_t val);

void rocks_logger_destroy(rocks_logger_t* logger);

void rocks_logger_log(rocks_logger_t* logger, int log_level, const char* msg_ptr, size_t msg_len);

void rocks_logger_flush(rocks_logger_t* logger);

void rocks_logger_set_log_level(rocks_logger_t* logger, int log_level);

int rocks_logger_get_log_level(rocks_logger_t* logger);

/* snapshot.h */

const rocks_snapshot_t* rocks_create_snapshot(rocks_db_t* db);

void rocks_snapshot_destroy(rocks_snapshot_t* snapshot);

void rocks_release_snapshot(rocks_db_t* db, const rocks_snapshot_t* snapshot);

uint64_t rocks_snapshot_get_sequence_number(rocks_snapshot_t* snapshot);
/* iterator.h */

/* write_batch.h */

rocks_writebatch_t* rocks_writebatch_create();

rocks_writebatch_t* rocks_writebatch_create_with_reserved_bytes(size_t size);

void rocks_writebatch_destroy(rocks_writebatch_t* b);

void rocks_writebatch_clear(rocks_writebatch_t* b);

int rocks_writebatch_count(rocks_writebatch_t* b);

void rocks_writebatch_put(rocks_writebatch_t* b, const char* key, size_t klen, const char* val, size_t vlen);

void rocks_writebatch_put_cf(rocks_writebatch_t* b, rocks_column_family_handle_t* column_family, const char* key,
                             size_t klen, const char* val, size_t vlen);

void rocks_writebatch_putv(rocks_writebatch_t* b, int num_keys, const char* const* keys_list,
                           const size_t* keys_list_sizes, int num_values, const char* const* values_list,
                           const size_t* values_list_sizes);

void rocks_writebatch_putv_cf(rocks_writebatch_t* b, rocks_column_family_handle_t* column_family, int num_keys,
                              const char* const* keys_list, const size_t* keys_list_sizes, int num_values,
                              const char* const* values_list, const size_t* values_list_sizes);

void rocks_writebatch_putv_coerce(rocks_writebatch_t* b, const void* key_parts, int num_keys, const void* value_parts,
                                  int num_values);

void rocks_writebatch_putv_cf_coerce(rocks_writebatch_t* b, rocks_column_family_handle_t* column_family,
                                     const void* key_parts, int num_keys, const void* value_parts, int num_values);

void rocks_writebatch_merge(rocks_writebatch_t* b, const char* key, size_t klen, const char* val, size_t vlen);

void rocks_writebatch_merge_cf(rocks_writebatch_t* b, rocks_column_family_handle_t* column_family, const char* key,
                               size_t klen, const char* val, size_t vlen);

void rocks_writebatch_mergev(rocks_writebatch_t* b, int num_keys, const char* const* keys_list,
                             const size_t* keys_list_sizes, int num_values, const char* const* values_list,
                             const size_t* values_list_sizes);

void rocks_writebatch_mergev_cf(rocks_writebatch_t* b, rocks_column_family_handle_t* column_family, int num_keys,
                                const char* const* keys_list, const size_t* keys_list_sizes, int num_values,
                                const char* const* values_list, const size_t* values_list_sizes);

void rocks_writebatch_mergev_coerce(rocks_writebatch_t* b, const void* key_parts, int num_keys, const void* value_parts,
                                    int num_values);

void rocks_writebatch_mergev_cf_coerce(rocks_writebatch_t* b, rocks_column_family_handle_t* column_family,
                                       const void* key_parts, int num_keys, const void* value_parts, int num_values);

void rocks_writebatch_delete(rocks_writebatch_t* b, const char* key, size_t klen);

void rocks_writebatch_delete_cf(rocks_writebatch_t* b, rocks_column_family_handle_t* column_family, const char* key,
                                size_t klen);

void rocks_writebatch_deletev(rocks_writebatch_t* b, int num_keys, const char* const* keys_list,
                              const size_t* keys_list_sizes);

void rocks_writebatch_deletev_cf(rocks_writebatch_t* b, rocks_column_family_handle_t* column_family, int num_keys,
                                 const char* const* keys_list, const size_t* keys_list_sizes);

void rocks_writebatch_deletev_coerce(rocks_writebatch_t* b, const void* key_parts, int num_keys);

void rocks_writebatch_deletev_cf_coerce(rocks_writebatch_t* b, rocks_column_family_handle_t* column_family,
                                        const void* key_parts, int num_keys);

void rocks_writebatch_single_deletev_coerce(rocks_writebatch_t* b, const void* key_parts, int num_keys);

void rocks_writebatch_single_deletev_cf_coerce(rocks_writebatch_t* b, rocks_column_family_handle_t* column_family,
                                               const void* key_parts, int num_keys);

void rocks_writebatch_single_delete(rocks_writebatch_t* b, const char* key, size_t klen);

void rocks_writebatch_single_delete_cf(rocks_writebatch_t* b, rocks_column_family_handle_t* column_family,
                                       const char* key, size_t klen);

void rocks_writebatch_delete_range(rocks_writebatch_t* b, const char* start_key, size_t start_key_len,
                                   const char* end_key, size_t end_key_len);

void rocks_writebatch_delete_range_cf(rocks_writebatch_t* b, rocks_column_family_handle_t* column_family,
                                      const char* start_key, size_t start_key_len, const char* end_key,
                                      size_t end_key_len);

void rocks_writebatch_deletev_range_cf_coerce(rocks_writebatch_t* b, rocks_column_family_handle_t* column_family,
                                              const void* begin_key_parts, int num_begin_keys,
                                              const void* end_key_parts, int num_end_keys);

void rocks_writebatch_put_log_data(rocks_writebatch_t* b, const char* blob, size_t len);

void rocks_writebatch_iterate(rocks_writebatch_t* b, void* trait_obj, rocks_status_t** status);

const char* rocks_writebatch_data(rocks_writebatch_t* b, size_t* size);

void rocks_writebatch_set_save_point(rocks_writebatch_t* b);

void rocks_writebatch_rollback_to_save_point(rocks_writebatch_t* b, rocks_status_t** status);

void rocks_writebatch_pop_save_point(rocks_writebatch_t* b, rocks_status_t** status);

unsigned char rocks_writebatch_has_put(rocks_writebatch_t* b);
unsigned char rocks_writebatch_has_delete(rocks_writebatch_t* b);
unsigned char rocks_writebatch_has_single_delete(rocks_writebatch_t* b);
unsigned char rocks_writebatch_has_delete_range(rocks_writebatch_t* b);
unsigned char rocks_writebatch_has_merge(rocks_writebatch_t* b);
unsigned char rocks_writebatch_has_begin_prepare(rocks_writebatch_t* b);
unsigned char rocks_writebatch_has_end_prepare(rocks_writebatch_t* b);
unsigned char rocks_writebatch_has_commit(rocks_writebatch_t* b);
unsigned char rocks_writebatch_has_rollback(rocks_writebatch_t* b);

rocks_writebatch_t* rocks_writebatch_copy(rocks_writebatch_t* b);
rocks_raw_writebatch_t* rocks_writebatch_get_writebatch(rocks_writebatch_t* b);

/* table */

rocks_plain_table_options_t* rocks_plain_table_options_create();

void rocks_plain_table_options_destroy(rocks_plain_table_options_t* options);

void rocks_plain_table_options_set_user_key_len(rocks_plain_table_options_t* options, uint32_t val);
void rocks_plain_table_options_set_bloom_bits_per_key(rocks_plain_table_options_t* options, int val);
void rocks_plain_table_options_set_hash_table_ratio(rocks_plain_table_options_t* options, double val);
void rocks_plain_table_options_set_index_sparseness(rocks_plain_table_options_t* options, size_t val);
void rocks_plain_table_options_set_huge_page_tlb_size(rocks_plain_table_options_t* options, size_t val);
void rocks_plain_table_options_set_encoding_type(rocks_plain_table_options_t* options, char val);
void rocks_plain_table_options_set_full_scan_mode(rocks_plain_table_options_t* options, unsigned char val);
void rocks_plain_table_options_set_store_index_in_file(rocks_plain_table_options_t* options, unsigned char val);

rocks_block_based_table_options_t* rocks_block_based_table_options_create();

void rocks_block_based_table_options_destroy(rocks_block_based_table_options_t* options);

// flush_block_policy_factory

void rocks_block_based_table_options_set_cache_index_and_filter_blocks(rocks_block_based_table_options_t* options,
                                                                       unsigned char val);
void rocks_block_based_table_options_set_cache_index_and_filter_blocks_with_high_priority(
    rocks_block_based_table_options_t* options, unsigned char val);
void rocks_block_based_table_options_set_pin_l0_filter_and_index_blocks_in_cache(
    rocks_block_based_table_options_t* options, unsigned char v);
void rocks_block_based_table_options_set_index_type(rocks_block_based_table_options_t* options, int v);
void rocks_block_based_table_options_set_hash_index_allow_collision(rocks_block_based_table_options_t* options,
                                                                    unsigned char v);
// checksum
void rocks_block_based_table_options_set_no_block_cache(rocks_block_based_table_options_t* options,
                                                        unsigned char no_block_cache);
void rocks_block_based_table_options_set_block_cache(rocks_block_based_table_options_t* options,
                                                     rocks_cache_t* block_cache);
void rocks_block_based_table_options_set_persistent_cache(rocks_block_based_table_options_t* options,
                                                          rocks_persistent_cache_t* cache);
void rocks_block_based_table_options_set_block_cache_compressed(rocks_block_based_table_options_t* options,
                                                                rocks_cache_t* block_cache_compressed);
void rocks_block_based_table_options_set_block_size(rocks_block_based_table_options_t* options, size_t block_size);
void rocks_block_based_table_options_set_block_size_deviation(rocks_block_based_table_options_t* options,
                                                              int block_size_deviation);
void rocks_block_based_table_options_set_block_restart_interval(rocks_block_based_table_options_t* options,
                                                                int block_restart_interval);
void rocks_block_based_table_options_set_index_block_restart_interval(rocks_block_based_table_options_t* options,
                                                                      int val);

void rocks_block_based_table_options_set_metadata_block_size(rocks_block_based_table_options_t* options, uint64_t val);

void rocks_block_based_table_options_set_partition_filters(rocks_block_based_table_options_t* options,
                                                           unsigned char val);
void rocks_block_based_table_options_set_use_delta_encoding(rocks_block_based_table_options_t* options,
                                                            unsigned char no_block_cache);
void rocks_block_based_table_options_set_filter_policy(rocks_block_based_table_options_t* options,
                                                       rocks_raw_filterpolicy_t* policy);
void rocks_block_based_table_options_set_whole_key_filtering(rocks_block_based_table_options_t* options,
                                                             unsigned char v);
void rocks_block_based_table_options_set_verify_compression(rocks_block_based_table_options_t* options,
                                                            unsigned char v);
void rocks_block_based_table_options_set_read_amp_bytes_per_bit(rocks_block_based_table_options_t* options, uint32_t v);
void rocks_block_based_table_options_set_format_version(rocks_block_based_table_options_t* options, uint32_t v);

rocks_cuckoo_table_options_t* rocks_cuckoo_table_options_create();

void rocks_cuckoo_table_options_destroy(rocks_cuckoo_table_options_t* options);

void rocks_cuckoo_table_options_set_hash_table_ratio(rocks_cuckoo_table_options_t* options, double v);

void rocks_cuckoo_table_options_set_max_search_depth(rocks_cuckoo_table_options_t* options, uint32_t v);

void rocks_cuckoo_table_options_set_cuckoo_block_size(rocks_cuckoo_table_options_t* options, uint32_t v);

void rocks_cuckoo_table_options_set_identity_as_first_hash(rocks_cuckoo_table_options_t* options, unsigned char v);

void rocks_cuckoo_table_options_set_use_module_hash(rocks_cuckoo_table_options_t* options, unsigned char v);

/* iterator */
void rocks_iter_destroy(rocks_iterator_t* iter);

unsigned char rocks_iter_valid(const rocks_iterator_t* iter);

void rocks_iter_seek_to_first(rocks_iterator_t* iter);

void rocks_iter_seek_to_last(rocks_iterator_t* iter);

void rocks_iter_seek(rocks_iterator_t* iter, const char* k, size_t klen);

void rocks_iter_seek_for_prev(rocks_iterator_t* iter, const char* k, size_t klen);

void rocks_iter_next(rocks_iterator_t* iter);

void rocks_iter_prev(rocks_iterator_t* iter);

const char* rocks_iter_key(const rocks_iterator_t* iter, size_t* klen);

const char* rocks_iter_value(const rocks_iterator_t* iter, size_t* vlen);

void rocks_iter_get_status(const rocks_iterator_t* iter, rocks_status_t** status);

void rocks_iter_get_property(const rocks_iterator_t* iter, const char* prop, size_t prop_len, void* value,
                             rocks_status_t** status);

rocks_iterator_t* rocks_new_empty_iterator();

/* filter_policy */
rocks_raw_filterpolicy_t* rocks_raw_filterpolicy_new_bloomfilter(int bits_per_key,
                                                                 unsigned char use_block_based_builder);
void rocks_raw_filterpolicy_destroy(rocks_raw_filterpolicy_t* cache);

/* cache */
rocks_cache_t* rocks_cache_create_lru(size_t capacity, int num_shard_bits, char strict_capacity_limit,
                                      double high_pri_pool_ratio);

rocks_cache_t* rocks_cache_create_clock(size_t capacity, int num_shard_bits, char strict_capacity_limit);

void rocks_cache_destroy(rocks_cache_t* cache);

void rocks_cache_set_capacity(rocks_cache_t* cache, size_t capacity);

size_t rocks_cache_get_capacity(rocks_cache_t* cache);

size_t rocks_cache_get_usage(rocks_cache_t* cache);

size_t rocks_cache_get_pinned_usage(rocks_cache_t* cache);

const char* rocks_cache_name(rocks_cache_t* cache);

/* persistent_cache */
rocks_persistent_cache_t* rocks_new_persistent_cache(const rocks_env_t* env, const char* path, size_t path_len,
                                                     uint64_t size, const rocks_logger_t* log,
                                                     unsigned char optimized_for_nvm, rocks_status_t** status);

void rocks_persistent_cache_destroy(rocks_persistent_cache_t* cache);
rocks_persistent_cache_t* rocks_persistent_cache_clone(rocks_persistent_cache_t* cache);

cxx_string_t* rocks_persistent_cache_get_printable_options(rocks_persistent_cache_t* cache);

/* sst_file_writer */
rocks_external_sst_file_info_t* rocks_external_sst_file_info_create();
void rocks_external_sst_file_info_destroy(rocks_external_sst_file_info_t* info);

const char* rocks_external_sst_file_info_get_file_path(rocks_external_sst_file_info_t* info, size_t* len);
const char* rocks_external_sst_file_info_get_smallest_key(rocks_external_sst_file_info_t* info, size_t* len);
const char* rocks_external_sst_file_info_get_largest_key(rocks_external_sst_file_info_t* info, size_t* len);
uint64_t rocks_external_sst_file_info_get_sequence_number(rocks_external_sst_file_info_t* info);
uint64_t rocks_external_sst_file_info_get_file_size(rocks_external_sst_file_info_t* info);
uint64_t rocks_external_sst_file_info_get_num_entries(rocks_external_sst_file_info_t* info);
int32_t rocks_external_sst_file_info_get_version(rocks_external_sst_file_info_t* info);

rocks_sst_file_writer_t* rocks_sst_file_writer_create_from_c_comparator(
    const rocks_envoptions_t* env_options, const rocks_options_t* options,
    const rocks_c_comparator_t* comparator, /* avoid export rocksdb::Comparator type */
    rocks_column_family_handle_t* column_family, unsigned char invalidate_page_cache);

rocks_sst_file_writer_t* rocks_sst_file_writer_create_from_rust_comparator(const rocks_envoptions_t* env_options,
                                                                           const rocks_options_t* options,
                                                                           const void* comparator,
                                                                           rocks_column_family_handle_t* column_family,
                                                                           unsigned char invalidate_page_cache);

void rocks_sst_file_writer_destroy(rocks_sst_file_writer_t* writer);

void rocks_sst_file_writer_open(rocks_sst_file_writer_t* writer, const char* file_path, const size_t file_path_len,
                                rocks_status_t** status);

void rocks_sst_file_writer_put(rocks_sst_file_writer_t* writer, const char* key, const size_t key_len,
                               const char* value, const size_t value_len, rocks_status_t** status);

void rocks_sst_file_writer_merge(rocks_sst_file_writer_t* writer, const char* key, const size_t key_len,
                                 const char* value, const size_t value_len, rocks_status_t** status);

void rocks_sst_file_writer_delete(rocks_sst_file_writer_t* writer, const char* key, const size_t key_len,
                                  rocks_status_t** status);

void rocks_sst_file_writer_finish(rocks_sst_file_writer_t* writer, rocks_external_sst_file_info_t* info,
                                  rocks_status_t** status);

uint64_t rocks_sst_file_writer_file_size(rocks_sst_file_writer_t* writer);

/* comparator */
/* avoid export rocksdb::Comparator type */
const rocks_c_comparator_t* rocks_comparator_bytewise();
const rocks_c_comparator_t* rocks_comparator_bytewise_reversed();

/* version */
int rocks_version_major();
int rocks_version_minor();
int rocks_version_patch();

/* db_dump_tool */
rocks_dump_options_t* rocks_dump_options_create();

void rocks_dump_options_destroy(rocks_dump_options_t* options);

void rocks_dump_options_set_db_path(rocks_dump_options_t* opt, const char* path, const size_t path_len);

void rocks_dump_options_set_dump_location(rocks_dump_options_t* opt, const char* path, const size_t path_len);

void rocks_dump_options_set_anonymous(rocks_dump_options_t* opt, unsigned char v);

unsigned char rocks_db_dump_tool_run(rocks_dump_options_t* dump_options, rocks_options_t* options);

rocks_undump_options_t* rocks_undump_options_create();

void rocks_undump_options_destroy(rocks_undump_options_t* options);

void rocks_undump_options_set_db_path(rocks_undump_options_t* opt, const char* path, const size_t path_len);

void rocks_undump_options_set_dump_location(rocks_undump_options_t* opt, const char* path, const size_t path_len);

void rocks_undump_options_set_compact_db(rocks_undump_options_t* opt, unsigned char v);

unsigned char rocks_db_undump_tool_run(rocks_undump_options_t* undump_options, rocks_options_t* options);

/* perf_level */
void rocks_set_perf_level(unsigned char level);
unsigned char rocks_get_perf_level();

/* iostats_context */
rocks_iostats_context_t* rocks_get_iostats_context();
void rocks_iostats_context_reset(rocks_iostats_context_t* ctx);
void rocks_iostats_context_to_string(const rocks_iostats_context_t* ctx, unsigned char exclude_zero_counters, void* s);

/* perf_context */
rocks_perf_context_t* rocks_get_perf_context();
void rocks_perf_context_reset(rocks_perf_context_t* ctx);
void rocks_perf_context_to_string(const rocks_perf_context_t* ctx, unsigned char exclude_zero_counters, void* s);

/* statistics */
rocks_statistics_t* rocks_statistics_create();

// FIXME: is this naming right?
rocks_statistics_t* rocks_statistics_copy(rocks_statistics_t* stat);

void rocks_statistics_destroy(rocks_statistics_t* stat);

uint64_t rocks_statistics_get_ticker_count(rocks_statistics_t* stat, uint32_t tickerType);

void rocks_statistics_histogram_data(rocks_statistics_t* stat, uint32_t type, rocks_histogram_data_t* const data);

void rocks_statistics_get_histogram_string(rocks_statistics_t* stat, uint32_t type,
                                           void* str);  // *mut String

void rocks_statistics_record_tick(rocks_statistics_t* stat, uint32_t tickerType, uint64_t count);

void rocks_statistics_set_ticker_count(rocks_statistics_t* stat, uint32_t tickerType, uint64_t count);

uint64_t rocks_statistics_get_and_reset_ticker_count(rocks_statistics_t* stat, uint32_t tickerType);

void rocks_statistics_measure_time(rocks_statistics_t* stat, uint32_t histogramType, uint64_t time);

void rocks_statistics_to_string(rocks_statistics_t* stat, void* str); /* *mut String */

unsigned char rocks_statistics_hist_enabled_for_type(rocks_statistics_t* stat, uint32_t type);

/* metadata */
int rocks_livefiles_count(const rocks_livefiles_t* lf);

const char* rocks_livefiles_name(const rocks_livefiles_t* lf, int index);

const char* rocks_livefiles_column_family_name(const rocks_livefiles_t* lf, int index);
const char* rocks_livefiles_db_path(const rocks_livefiles_t* lf, int index);

uint64_t rocks_livefiles_smallest_seqno(const rocks_livefiles_t* lf, int index);

uint64_t rocks_livefiles_largest_seqno(const rocks_livefiles_t* lf, int index);

int rocks_livefiles_level(const rocks_livefiles_t* lf, int index);

size_t rocks_livefiles_size(const rocks_livefiles_t* lf, int index);

const char* rocks_livefiles_smallestkey(const rocks_livefiles_t* lf, int index, size_t* size);

const char* rocks_livefiles_largestkey(const rocks_livefiles_t* lf, int index, size_t* size);

unsigned char rocks_livefiles_being_compacted(const rocks_livefiles_t* lf, int index);

extern void rocks_livefiles_destroy(const rocks_livefiles_t* lf);

uint64_t rocks_column_family_metadata_size(const rocks_column_family_metadata_t* meta);
size_t rocks_column_family_metadata_file_count(const rocks_column_family_metadata_t* meta);
const char* rocks_column_family_metadata_name(const rocks_column_family_metadata_t* meta);
int rocks_column_family_metadata_levels_count(const rocks_column_family_metadata_t* meta);
int rocks_column_family_metadata_levels_level(const rocks_column_family_metadata_t* meta, int level);
uint64_t rocks_column_family_metadata_levels_size(const rocks_column_family_metadata_t* meta, int level);
int rocks_column_family_metadata_levels_files_count(const rocks_column_family_metadata_t* meta, int level);

size_t rocks_column_family_metadata_levels_files_size(const rocks_column_family_metadata_t* meta, int level,
                                                      int file_index);
const char* rocks_column_family_metadata_levels_files_name(const rocks_column_family_metadata_t* meta, int level,
                                                           int file_index);
const char* rocks_column_family_metadata_levels_files_db_path(const rocks_column_family_metadata_t* meta, int level,
                                                              int file_index);
uint64_t rocks_column_family_metadata_levels_files_smallest_seqno(const rocks_column_family_metadata_t* meta, int level,
                                                                  int file_index);
uint64_t rocks_column_family_metadata_levels_files_largest_seqno(const rocks_column_family_metadata_t* meta, int level,
                                                                 int file_index);
const char* rocks_column_family_metadata_levels_files_smallestkey(const rocks_column_family_metadata_t* meta, int level,
                                                                  int file_index, size_t* size);
const char* rocks_column_family_metadata_levels_files_largestkey(const rocks_column_family_metadata_t* meta, int level,
                                                                 int file_index, size_t* size);
unsigned char rocks_column_family_metadata_levels_files_being_compacted(const rocks_column_family_metadata_t* meta,
                                                                        int level, int file_index);

extern void rocks_column_family_metadata_destroy(const rocks_column_family_metadata_t* meta);

/* universal_compaction */
rocks_universal_compaction_options_t* rocks_universal_compaction_options_create();

void rocks_universal_compaction_options_set_size_ratio(rocks_universal_compaction_options_t* uco, unsigned int ratio);

void rocks_universal_compaction_options_set_min_merge_width(rocks_universal_compaction_options_t* uco, unsigned int w);

void rocks_universal_compaction_options_set_max_merge_width(rocks_universal_compaction_options_t* uco, unsigned int w);

void rocks_universal_compaction_options_set_max_size_amplification_percent(rocks_universal_compaction_options_t* uco,
                                                                           unsigned int p);

void rocks_universal_compaction_options_set_compression_size_percent(rocks_universal_compaction_options_t* uco, int p);

void rocks_universal_compaction_options_set_stop_style(rocks_universal_compaction_options_t* uco, int style);

void rocks_universal_compaction_options_destroy(rocks_universal_compaction_options_t* uco);

void rocks_universal_compaction_options_set_allow_trivial_move(rocks_universal_compaction_options_t* uco,
                                                               unsigned char val);

/* transaction_log */
void rocks_logfiles_destroy(rocks_logfiles_t* files);

size_t rocks_logfiles_size(rocks_logfiles_t* files);
void rocks_logfiles_nth_path_name(rocks_logfiles_t* files, size_t nth, void* s);
uint64_t rocks_logfiles_nth_log_number(rocks_logfiles_t* files, size_t nth);
int rocks_logfiles_nth_type(rocks_logfiles_t* files, size_t nth);
uint64_t rocks_logfiles_nth_start_sequence(rocks_logfiles_t* files, size_t nth);
uint64_t rocks_logfiles_nth_file_size(rocks_logfiles_t* files, size_t nth);

void rocks_transaction_log_iterator_destory(rocks_transaction_log_iterator_t* it);
unsigned char rocks_transaction_log_iterator_valid(rocks_transaction_log_iterator_t* it);
void rocks_transaction_log_iterator_next(rocks_transaction_log_iterator_t* it);
void rocks_transaction_log_iterator_status(rocks_transaction_log_iterator_t* it, rocks_status_t** status);
rocks_writebatch_t* rocks_transaction_log_iterator_get_batch(rocks_transaction_log_iterator_t* it, uint64_t* seq_no);

/* convenience */
int* rocks_get_supported_compressions(size_t* len);
void rocks_get_supported_compressions_destroy(int* ptr);
void rocks_cancel_all_background_work(rocks_db_t* db, unsigned char wait);
void rocks_db_delete_files_in_range(rocks_db_t* db, rocks_column_family_handle_t* column_family, const char* begin_ptr,
                                    size_t begin_len, const char* end_ptr, size_t end_len, rocks_status_t** status);
// cxx_string_destroy must be called for following
cxx_string_t* rocks_get_string_from_dboptions(rocks_dboptions_t* opts);
cxx_string_t* rocks_get_string_from_cfoptions(rocks_cfoptions_t* opts);

/* table_properties */
void rocks_table_props_collection_destroy(rocks_table_props_collection_t* coll);

void rocks_table_props_destroy(rocks_table_props_t* props);

void rocks_table_props_collection_iter_destroy(rocks_table_props_collection_iter_t* it);

void rocks_user_collected_props_iter_destroy(rocks_user_collected_props_iter_t* it);

size_t rocks_table_props_collection_size(rocks_table_props_collection_t* coll);

rocks_table_props_t* rocks_table_props_collection_at(rocks_table_props_collection_t* coll, const char* key_ptr,
                                                     size_t key_len);

rocks_table_props_collection_iter_t* rocks_table_props_collection_iter_create(rocks_table_props_collection_t* coll);

unsigned char rocks_table_props_collection_iter_next(rocks_table_props_collection_iter_t* it);

const char* rocks_table_props_collection_iter_key(rocks_table_props_collection_iter_t* it, size_t* len);

rocks_table_props_t* rocks_table_props_collection_iter_value(rocks_table_props_collection_iter_t* it);

uint64_t rocks_table_props_get_data_size(rocks_table_props_t* prop);
uint64_t rocks_table_props_get_index_size(rocks_table_props_t* prop);
uint64_t rocks_table_props_get_filter_size(rocks_table_props_t* prop);
uint64_t rocks_table_props_get_raw_key_size(rocks_table_props_t* prop);
uint64_t rocks_table_props_get_raw_value_size(rocks_table_props_t* prop);
uint64_t rocks_table_props_get_num_data_blocks(rocks_table_props_t* prop);
uint64_t rocks_table_props_get_num_entries(rocks_table_props_t* prop);
uint64_t rocks_table_props_get_format_version(rocks_table_props_t* prop);
uint64_t rocks_table_props_get_fixed_key_len(rocks_table_props_t* prop);
uint32_t rocks_table_props_get_column_family_id(rocks_table_props_t* prop);
const char* rocks_table_props_get_column_family_name(rocks_table_props_t* prop, size_t* len);
const char* rocks_table_props_get_filter_policy_name(rocks_table_props_t* prop, size_t* len);
const char* rocks_table_props_get_comparator_name(rocks_table_props_t* prop, size_t* len);
const char* rocks_table_props_get_merge_operator_name(rocks_table_props_t* prop, size_t* len);
const char* rocks_table_props_get_prefix_extractor_name(rocks_table_props_t* prop, size_t* len);
const char* rocks_table_props_get_property_collectors_names(rocks_table_props_t* prop, size_t* len);
const char* rocks_table_props_get_compression_name(rocks_table_props_t* prop, size_t* len);

void rocks_table_props_to_string(rocks_table_props_t* prop, void* s);

rocks_user_collected_props_t* rocks_table_props_get_user_collected_properties(rocks_table_props_t* prop);

rocks_user_collected_props_t* rocks_table_props_get_readable_properties(rocks_table_props_t* prop);

void rocks_user_collected_props_insert(rocks_user_collected_props_t* prop, const char* key_ptr, size_t key_len,
                                       const char* val_ptr, size_t val_len);

size_t rocks_user_collected_props_size(rocks_user_collected_props_t* prop);

const char* rocks_user_collected_props_at(rocks_user_collected_props_t* prop, const char* key_ptr, size_t key_len,
                                          size_t* value_len);

rocks_user_collected_props_iter_t* rocks_user_collected_props_iter_create(rocks_user_collected_props_t* prop);

unsigned char rocks_user_collected_props_iter_next(rocks_user_collected_props_iter_t* it);

const char* rocks_user_collected_props_iter_key(rocks_user_collected_props_iter_t* it, size_t* len);

const char* rocks_user_collected_props_iter_value(rocks_user_collected_props_iter_t* it, size_t* len);

/* write_buffer_manager */
rocks_write_buffer_manager_t* rocks_write_buffer_manager_create(size_t buffer_size);

void rocks_write_buffer_manager_destroy(rocks_write_buffer_manager_t* manager);

unsigned char rocks_write_buffer_manager_enabled(rocks_write_buffer_manager_t* manager);
size_t rocks_write_buffer_manager_memory_usage(rocks_write_buffer_manager_t* manager);
size_t rocks_write_buffer_manager_buffer_size(rocks_write_buffer_manager_t* manager);

/* debug */
rocks_key_version_collection_t* rocks_db_get_all_key_versions(rocks_db_t* db, const char* begin_key,
                                                              size_t begin_keylen, const char* end_key,
                                                              size_t end_keylen, rocks_status_t** status);

void rocks_key_version_collection_destroy(rocks_key_version_collection_t* coll);
size_t rocks_key_version_collection_size(rocks_key_version_collection_t* coll);
rocks_key_version_t* rocks_key_version_collection_nth(rocks_key_version_collection_t* coll, size_t index);

const char* rocks_key_version_user_key(const rocks_key_version_t* ver, size_t* len);
const char* rocks_key_version_value(const rocks_key_version_t* ver, size_t* len);
uint64_t rocks_key_version_sequence_numer(const rocks_key_version_t* ver);
int rocks_key_version_type(const rocks_key_version_t* ver);

/* listener */
/* change pointer type */
const char* rocks_flush_job_info_get_cf_name(const rocks_flush_job_info_t* info, size_t* len);

const char* rocks_flush_job_info_get_file_path(const rocks_flush_job_info_t* info, size_t* len);

uint64_t rocks_flush_job_info_get_thread_id(const rocks_flush_job_info_t* info);

uint64_t rocks_flush_job_info_get_job_id(const rocks_flush_job_info_t* info);

unsigned char rocks_flush_job_info_get_triggered_writes_slowdown(const rocks_flush_job_info_t* info);

unsigned char rocks_flush_job_info_get_triggered_writes_stop(const rocks_flush_job_info_t* info);

uint64_t rocks_flush_job_info_get_smallest_seqno(const rocks_flush_job_info_t* info);

uint64_t rocks_flush_job_info_get_largest_seqno(const rocks_flush_job_info_t* info);

rocks_table_props_t* rocks_flush_job_info_get_table_properties(const rocks_flush_job_info_t* info);

const char* rocks_table_file_deletion_info_get_db_name(const rocks_table_file_deletion_info_t* info, size_t* len);

const char* rocks_table_file_deletion_info_get_file_path(const rocks_table_file_deletion_info_t* info, size_t* len);

uint64_t rocks_table_file_deletion_info_get_job_id(const rocks_table_file_deletion_info_t* info);

void rocks_table_file_deletion_info_get_status(const rocks_table_file_deletion_info_t* info, rocks_status_t** status);

const char* rocks_compaction_job_info_get_cf_name(const rocks_compaction_job_info_t* info, size_t* len);
void rocks_compaction_job_info_get_status(const rocks_compaction_job_info_t* info, rocks_status_t** status);
uint64_t rocks_compaction_job_info_get_thread_id(const rocks_compaction_job_info_t* info);
int rocks_compaction_job_info_get_job_id(const rocks_compaction_job_info_t* info);
int rocks_compaction_job_info_get_base_input_level(const rocks_compaction_job_info_t* info);
int rocks_compaction_job_info_get_output_level(const rocks_compaction_job_info_t* info);

size_t rocks_compaction_job_info_get_input_files_num(const rocks_compaction_job_info_t* info);
// requires: files, sizes buf allocated with size acquired via above method
void rocks_compaction_job_info_get_input_files(const rocks_compaction_job_info_t* info, const char** files,
                                               size_t* sizes);

size_t rocks_compaction_job_info_get_output_files_num(const rocks_compaction_job_info_t* info);
// requires: files, sizes buf allocated with size acquired via above method
void rocks_compaction_job_info_get_output_files(const rocks_compaction_job_info_t* info, const char** files,
                                                size_t* sizes);

rocks_table_props_collection_t* rocks_compaction_job_info_get_table_properties(const rocks_compaction_job_info_t* info);
int rocks_compaction_job_info_get_compaction_reason(const rocks_compaction_job_info_t* info);
int rocks_compaction_job_info_get_compression(const rocks_compaction_job_info_t* info);
rocks_compaction_job_stats_t* rocks_compaction_job_info_get_stats(const rocks_compaction_job_info_t* info);

/* compaction_job_stats */
uint64_t rocks_compaction_job_stats_get_elapsed_micros(const rocks_compaction_job_stats_t* stats);
uint64_t rocks_compaction_job_stats_get_num_input_records(const rocks_compaction_job_stats_t* stats);
size_t rocks_compaction_job_stats_get_num_input_files(const rocks_compaction_job_stats_t* stats);
size_t rocks_compaction_job_stats_get_num_input_files_at_output_level(const rocks_compaction_job_stats_t* stats);
uint64_t rocks_compaction_job_stats_get_num_output_records(const rocks_compaction_job_stats_t* stats);
size_t rocks_compaction_job_stats_get_num_output_files(const rocks_compaction_job_stats_t* stats);
unsigned char rocks_compaction_job_stats_get_is_manual_compaction(const rocks_compaction_job_stats_t* stats);
uint64_t rocks_compaction_job_stats_get_total_input_bytes(const rocks_compaction_job_stats_t* stats);
uint64_t rocks_compaction_job_stats_get_total_output_bytes(const rocks_compaction_job_stats_t* stats);
uint64_t rocks_compaction_job_stats_get_num_records_replaced(const rocks_compaction_job_stats_t* stats);
uint64_t rocks_compaction_job_stats_get_total_input_raw_key_bytes(const rocks_compaction_job_stats_t* stats);
uint64_t rocks_compaction_job_stats_get_total_input_raw_value_bytes(const rocks_compaction_job_stats_t* stats);
uint64_t rocks_compaction_job_stats_get_num_input_deletion_records(const rocks_compaction_job_stats_t* stats);
uint64_t rocks_compaction_job_stats_get_num_expired_deletion_records(const rocks_compaction_job_stats_t* stats);
uint64_t rocks_compaction_job_stats_get_num_corrupt_keys(const rocks_compaction_job_stats_t* stats);
uint64_t rocks_compaction_job_stats_get_file_write_nanos(const rocks_compaction_job_stats_t* stats);
uint64_t rocks_compaction_job_stats_get_file_range_sync_nanos(const rocks_compaction_job_stats_t* stats);
uint64_t rocks_compaction_job_stats_get_file_fsync_nanos(const rocks_compaction_job_stats_t* stats);
uint64_t rocks_compaction_job_stats_get_file_prepare_write_nanos(const rocks_compaction_job_stats_t* stats);
const char* rocks_compaction_job_stats_get_smallest_output_key_prefix(const rocks_compaction_job_stats_t* stats,
                                                                      size_t* len);
const char* rocks_compaction_job_stats_get_largest_output_key_prefix(const rocks_compaction_job_stats_t* stats,
                                                                     size_t* len);
uint64_t rocks_compaction_job_stats_get_num_single_del_fallthru(const rocks_compaction_job_stats_t* stats);
uint64_t rocks_compaction_job_stats_get_num_single_del_mismatch(const rocks_compaction_job_stats_t* stats);

uint64_t rocks_table_file_creation_info_get_file_size(const rocks_table_file_creation_info_t* info);
rocks_table_props_t* rocks_table_file_creation_info_get_table_properties(const rocks_table_file_creation_info_t* info);
void rocks_table_file_creation_info_get_status(const rocks_table_file_creation_info_t* info, rocks_status_t** status);
const rocks_table_file_creation_brief_info_t* rocks_table_file_creation_info_get_brief_info(
    const rocks_table_file_creation_info_t* info);

const char* rocks_table_file_creation_brief_info_get_db_name(const rocks_table_file_creation_brief_info_t* info,
                                                             size_t* len);
const char* rocks_table_file_creation_brief_info_get_cf_name(const rocks_table_file_creation_brief_info_t* info,
                                                             size_t* len);
const char* rocks_table_file_creation_brief_info_get_file_path(const rocks_table_file_creation_brief_info_t* info,
                                                               size_t* len);
int rocks_table_file_creation_brief_info_get_job_id(const rocks_table_file_creation_brief_info_t* info);
int rocks_table_file_creation_brief_info_get_reason(const rocks_table_file_creation_brief_info_t* info);

const char* rocks_mem_table_info_get_cf_name(const rocks_mem_table_info_t* info, size_t* len);
uint64_t rocks_mem_table_info_get_first_seqno(const rocks_mem_table_info_t* info);
uint64_t rocks_mem_table_info_get_earliest_seqno(const rocks_mem_table_info_t* info);
uint64_t rocks_mem_table_info_get_num_entries(const rocks_mem_table_info_t* info);
uint64_t rocks_mem_table_info_get_num_deletes(const rocks_mem_table_info_t* info);

const char* rocks_external_file_ingestion_info_get_cf_name(const rocks_external_file_ingestion_info_t* info,
                                                           size_t* len);
const char* rocks_external_file_ingestion_info_get_external_file_path(const rocks_external_file_ingestion_info_t* info,
                                                                      size_t* len);
const char* rocks_external_file_ingestion_info_get_internal_file_path(const rocks_external_file_ingestion_info_t* info,
                                                                      size_t* len);
uint64_t rocks_external_file_ingestion_info_get_global_seqno(const rocks_external_file_ingestion_info_t* info);
rocks_table_props_t* rocks_external_file_ingestion_info_get_table_properties(
    const rocks_external_file_ingestion_info_t* info);

/* thread_status */
void rocks_thread_status_destroy(rocks_thread_status_t* status);

uint64_t rocks_thread_status_get_thread_id(const rocks_thread_status_t* status);
int rocks_thread_status_get_thread_type(const rocks_thread_status_t* status);
const char* rocks_thread_status_get_db_name(const rocks_thread_status_t* status, size_t* len);
const char* rocks_thread_status_get_cf_name(const rocks_thread_status_t* status, size_t* len);
int rocks_thread_status_get_operation_type(const rocks_thread_status_t* status);
uint64_t rocks_thread_status_get_op_elapsed_micros(const rocks_thread_status_t* status);
int rocks_thread_status_get_operation_stage(const rocks_thread_status_t* status);
const uint64_t* rocks_thread_status_get_op_properties(const rocks_thread_status_t* status, size_t* len);
int rocks_thread_status_get_state_type(const rocks_thread_status_t* status);

/* options_util */
rocks_column_family_descriptor_t** rocks_load_latest_options(const char* c_dbpath, rocks_dboptions_t* db_options,
                                                             size_t* cf_descs_len, rocks_status_t** status);
void rocks_load_options_destroy_cf_descs(rocks_column_family_descriptor_t** c_cf_descs, size_t len);

/* aux */
void free(void* p);

size_t cxx_vector_slice_size(const void* list);
const void* cxx_vector_slice_nth(const void* list, size_t n);

/* std::string */
void cxx_string_assign(cxx_string_t* s, const char* p, size_t len);
const char* cxx_string_data(const cxx_string_t* s);
size_t cxx_string_size(const cxx_string_t* s);
void cxx_string_destroy(cxx_string_t* s);

cxx_string_vector_t* cxx_string_vector_create();
void cxx_string_vector_destory(cxx_string_vector_t* v);
size_t cxx_string_vector_size(cxx_string_vector_t* v);
const char* cxx_string_vector_nth(cxx_string_vector_t* v, size_t index);
size_t cxx_string_vector_nth_size(cxx_string_vector_t* v, size_t index);

#ifdef __cplusplus
}
#endif
