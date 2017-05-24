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

  /* db.h */
  typedef struct rocks_column_family_descriptor_t rocks_column_family_descriptor_t ;
  typedef struct rocks_column_family_handle_t     rocks_column_family_handle_t     ;
  typedef struct rocks_db_t                       rocks_db_t                       ;

  /* options.h */
  typedef struct rocks_cfoptions_t       rocks_cfoptions_t;
  typedef struct rocks_dbpath_t                    rocks_dbpath_t;
  typedef struct rocks_dboptions_t                 rocks_dboptions_t;
  typedef struct rocks_options_t                   rocks_options_t ;
  typedef struct rocks_readoptions_t               rocks_readoptions_t;
  typedef struct rocks_writeoptions_t              rocks_writeoptions_t;
  typedef struct rocks_flushoptions_t              rocks_flushoptions_t;
  typedef struct rocks_compaction_options_t         rocks_compaction_options_t;
  typedef struct rocks_compactrange_options_t       rocks_compactrange_options_t;
  typedef struct rocks_ingestexternalfile_options_t rocks_ingestexternalfile_options_t;

  /* status.h */
  typedef struct rocks_status_t {
    int code;
    int sub_code;
    const char *state;
  } rocks_status_t;

  /* rate_limiter.h */
  typedef struct rocks_ratelimiter_t     rocks_ratelimiter_t;

  /* env */
  typedef struct rocks_envoptions_t      rocks_envoptions_t     ;
  typedef struct rocks_logger_t          rocks_logger_t         ;
  typedef struct rocks_env_t rocks_env_t;

  /* snapshot*/
  typedef struct rocks_snapshot_t        rocks_snapshot_t       ;

  /* iterator */
  typedef struct rocks_iterator_t rocks_iterator_t;

  /* write_batch */
  typedef struct rocks_writebatch_t rocks_writebatch_t;

  /* cache */
  typedef struct rocks_cache_t           rocks_cache_t;

  /* merge_operator.h */
  typedef struct rocks_associative_mergeoperator_t rocks_associative_mergeoperator_t;
  typedef struct rocks_mergeoperator_t rocks_mergeoperator_t;

  /* comparator.h */
  typedef struct rocks_comparator_t rocks_comparator_t; /* for rust trait object */

  typedef struct rocks_c_comparator_t rocks_c_comparator_t; /* for c */

  /* sst_file_writer.h */
  typedef struct rocks_sst_file_writer_t rocks_sst_file_writer_t;
  typedef struct rocks_external_sst_file_info_t rocks_external_sst_file_info_t;

  /* ****************************** functions ****************************** */
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

  void rocks_cfoptions_set_compaction_filter_factory(rocks_options_t* opt, rocks_compactionfilterfactory_t* factory);
  */

  void rocks_cfoptions_set_write_buffer_size(rocks_cfoptions_t* opt, size_t s);

  void rocks_cfoptions_set_compression(rocks_cfoptions_t* opt, int t);

  void rocks_cfoptions_set_bottommost_compression(rocks_cfoptions_t* opt, int t);

  void rocks_cfoptions_set_compression_options(rocks_cfoptions_t* opt, int w_bits,
                                               int level, int strategy,
                                               uint32_t max_dict_bytes);

  void rocks_cfoptions_set_level0_file_num_compaction_trigger(rocks_cfoptions_t* opt, int n);

  void rocks_cfoptions_set_prefix_extractor_by_trait(rocks_cfoptions_t* opt, void* trans_trait_obj);
  void rocks_cfoptions_set_prefix_extractor_fixed_prefix(rocks_cfoptions_t* opt, size_t prefix_len);
  void rocks_cfoptions_set_prefix_extractor_capped_prefix(rocks_cfoptions_t* opt, size_t cap_len);
  void rocks_cfoptions_set_prefix_extractor_noop(rocks_cfoptions_t* opt);

  void rocks_cfoptions_set_max_bytes_for_level_base(rocks_cfoptions_t* opt, uint64_t n);

  void rocks_cfoptions_set_disable_auto_compactions(rocks_cfoptions_t* opt, unsigned char disable);

  // rocks_cfoptions_set_table_factory()
  // table_factory
  void rocks_cfoptions_set_plain_table_factory(
                                             rocks_cfoptions_t *opt, uint32_t user_key_len, int bloom_bits_per_key,
                                             double hash_table_ratio, size_t index_sparseness);

  // via AdvancedColumnFamilyOptions

  void rocks_cfoptions_set_max_write_buffer_number(rocks_cfoptions_t* opt, int n);

  void rocks_cfoptions_set_min_write_buffer_number_to_merge(rocks_cfoptions_t* opt, int n);

  void rocks_cfoptions_set_max_write_buffer_number_to_maintain(rocks_cfoptions_t* opt, int n);

  void rocks_cfoptions_set_inplace_update_support(rocks_cfoptions_t* opt, unsigned char v);

  void rocks_cfoptions_set_inplace_update_num_locks(rocks_cfoptions_t* opt, size_t v);

  // inplace_callback

  void rocks_cfoptions_set_memtable_prefix_bloom_size_ratio(rocks_cfoptions_t* opt, double v);

  void rocks_cfoptions_set_memtable_huge_page_size(rocks_cfoptions_t* opt, size_t v);

  void rocks_cfoptions_set_memtable_insert_with_hint_prefix_extractor_by_trait(rocks_cfoptions_t* opt, void* trans_trait_obj);
  void rocks_cfoptions_set_memtable_insert_with_hint_prefix_extractor_fixed_prefix(rocks_cfoptions_t* opt, size_t prefix_len);
  void rocks_cfoptions_set_memtable_insert_with_hint_prefix_extractor_capped_prefix(rocks_cfoptions_t* opt, size_t cap_len);
  void rocks_cfoptions_set_memtable_insert_with_hint_prefix_extractor_noop(rocks_cfoptions_t* opt);

  void rocks_cfoptions_set_bloom_locality(rocks_cfoptions_t* opt, uint32_t v);

  void rocks_cfoptions_set_arena_block_size(rocks_cfoptions_t* opt, size_t v);

  void rocks_cfoptions_set_compression_per_level(rocks_cfoptions_t* opt,
                                                 const int* level_values,
                                                 size_t num_levels);

  void rocks_cfoptions_set_num_levels(rocks_cfoptions_t* opt, int n);

  void rocks_cfoptions_set_level0_slowdown_writes_trigger(rocks_cfoptions_t* opt, int n);

  void rocks_cfoptions_set_level0_stop_writes_trigger(rocks_cfoptions_t* opt, int n);

  void rocks_cfoptions_set_target_file_size_base(rocks_cfoptions_t* opt, uint64_t n);

  void rocks_cfoptions_set_target_file_size_multiplier(rocks_cfoptions_t* opt, int n);

  void rocks_cfoptions_set_level_compaction_dynamic_level_bytes(rocks_cfoptions_t* opt, unsigned char v);

  void rocks_cfoptions_set_max_bytes_for_level_multiplier(rocks_cfoptions_t* opt, double n);

  void rocks_cfoptions_set_max_bytes_for_level_multiplier_additional(
    rocks_cfoptions_t* opt, int* level_values, size_t num_levels);

  void rocks_cfoptions_set_max_compaction_bytes(rocks_cfoptions_t* opt,
                                                uint64_t n);

  void rocks_cfoptions_set_soft_pending_compaction_bytes_limit(rocks_cfoptions_t* opt, uint64_t v);

  void rocks_cfoptions_set_hard_pending_compaction_bytes_limit(rocks_cfoptions_t* opt, uint64_t v);

  void rocks_cfoptions_set_compaction_style(rocks_cfoptions_t *opt, int style);

  void rocks_cfoptions_set_compaction_pri(rocks_cfoptions_t *opt, int pri);

  /*
    void rocks_cfoptions_set_universal_compaction_options(rocks_cfoptions_t *opt,
                                                          rocks_universal_compaction_options_t *uco);
  */

  /*
  void rocks_cfoptions_set_fifo_compaction_options(
                                                 rocks_cfoptions_t* opt,
                                                 rocks_fifo_compaction_options_t* fifo);  }
  */

  void rocks_cfoptions_set_max_sequential_skip_in_iterations(rocks_cfoptions_t* opt, uint64_t v);

  // memtable_factory
  void rocks_cfoptions_set_memtable_vector_rep(rocks_cfoptions_t *opt);

  void rocks_cfoptions_set_hash_skip_list_rep(
    rocks_cfoptions_t *opt, size_t bucket_count,
    int32_t skiplist_height, int32_t skiplist_branching_factor);

  void rocks_cfoptions_set_hash_link_list_rep(rocks_cfoptions_t *opt, size_t bucket_count);

  /*
  void rocks_cfoptions_set_table_properties_collector_factories(rocks_cfoptions_t *opt,
                                                                rocks_table_properties_collector_factory_t* factories,
                                                                size_t n);
  */

  void rocks_cfoptions_set_max_successive_merges(rocks_cfoptions_t* opt, size_t v);

  void rocks_cfoptions_set_optimize_filters_for_hits(rocks_cfoptions_t* opt, unsigned char v);

  void rocks_cfoptions_set_paranoid_file_checks(rocks_cfoptions_t* opt, unsigned char v);

  void rocks_cfoptions_set_force_consistency_checks(rocks_cfoptions_t* opt, unsigned char v);

  void rocks_cfoptions_set_report_bg_io_stats(rocks_cfoptions_t* opt, unsigned char v);

  // dboptions

  void rocks_dboptions_optimize_for_small_db(rocks_dboptions_t* opt);

  void rocks_dboptions_increase_parallelism(rocks_dboptions_t* opt, int total_threads);

  void rocks_dboptions_set_create_if_missing(
                                           rocks_dboptions_t* opt, unsigned char v);

  void rocks_dboptions_set_create_missing_column_families(
                                                        rocks_dboptions_t* opt, unsigned char v);

  void rocks_dboptions_set_error_if_exists(
                                         rocks_dboptions_t* opt, unsigned char v);

  void rocks_dboptions_set_paranoid_checks(rocks_dboptions_t* opt, unsigned char v);

  void rocks_dboptions_set_env(rocks_dboptions_t* opt, rocks_env_t* env);

  void rocks_dboptions_set_ratelimiter(rocks_dboptions_t* opt, rocks_ratelimiter_t* limiter);

  // void rocks_dboptions_set_sst_file_manager(rocks_dboptions_t* opt, rocks_sst_file_manager_t* manager);

  void rocks_dboptions_set_info_log(rocks_dboptions_t* opt, rocks_logger_t* l);

  void rocks_dboptions_set_info_log_level(rocks_dboptions_t* opt, int v);

  void rocks_dboptions_set_max_open_files(rocks_dboptions_t* opt, int n);

  void rocks_dboptions_set_max_file_opening_threads(rocks_dboptions_t* opt, int n);

  void rocks_dboptions_set_max_total_wal_size(rocks_dboptions_t* opt, uint64_t n);

  void rocks_dboptions_enable_statistics(rocks_dboptions_t* opt);

  void rocks_dboptions_set_use_fsync(rocks_dboptions_t* opt, unsigned char use_fsync);

  void rocks_dboptions_set_db_paths(rocks_dboptions_t* opt,
                                    const char* const* paths,
                                    const size_t* path_lens,
                                    const uint64_t* target_sizes,
                                    int size);

  void rocks_dboptions_set_db_log_dir(rocks_dboptions_t* opt, const char* db_log_dir, size_t len);

  void rocks_dboptions_set_wal_dir(rocks_dboptions_t* opt, const char* v, size_t len);

  void rocks_dboptions_set_delete_obsolete_files_period_micros(rocks_dboptions_t* opt, uint64_t v);

  void rocks_dboptions_set_base_background_compactions(rocks_dboptions_t* opt, int n);

  void rocks_dboptions_set_max_background_compactions(rocks_dboptions_t* opt, int n);

  void rocks_dboptions_set_max_subcompactions(rocks_dboptions_t* opt, uint32_t n);

  void rocks_dboptions_set_max_background_flushes(rocks_dboptions_t* opt, int n);

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

  void rocks_dboptions_set_use_direct_writes(rocks_dboptions_t* opt, unsigned char v);

  void rocks_dboptions_set_allow_fallocate(rocks_dboptions_t* opt, unsigned char v);

  void rocks_dboptions_set_is_fd_close_on_exec(rocks_dboptions_t* opt, unsigned char v);

  void rocks_dboptions_set_stats_dump_period_sec(rocks_dboptions_t* opt, unsigned int v);

  void rocks_dboptions_set_advise_random_on_open(rocks_dboptions_t* opt, unsigned char v);

  void rocks_dboptions_set_db_write_buffer_size(rocks_dboptions_t* opt, size_t s);

  /*
  void rocks_dboptions_set_write_buffer_manager(rocks_dboptions_t* opt,
                                              rocks_write_buffer_manager_t* manager);
  */

  void rocks_dboptions_set_access_hint_on_compaction_start(rocks_dboptions_t* opt, int v);

  void rocks_dboptions_set_new_table_reader_for_compaction_inputs(
                                               rocks_dboptions_t* opt, unsigned char v);
  void rocks_dboptions_set_compaction_readahead_size(
                                               rocks_dboptions_t* opt, size_t s);
  void rocks_dboptions_set_random_access_max_buffer_size(rocks_dboptions_t* opt,
                                              size_t s);
  void rocks_dboptions_set_writable_file_max_buffer_size(rocks_dboptions_t* opt,
                                                       size_t s);
  void rocks_dboptions_set_use_adaptive_mutex(
                                            rocks_dboptions_t* opt, unsigned char v);
  void rocks_dboptions_set_bytes_per_sync(
                                        rocks_dboptions_t* opt, uint64_t v);
  void rocks_dboptions_set_wal_bytes_per_sync(
                                        rocks_dboptions_t* opt, uint64_t v);
  /*
  void rocks_dboptions_set_listeners(rocks_dboptions_t* opt, rocks_event_listener_t* listeners, size_t n);
  */

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
  void rocks_dboptions_set_wal_filter(rocks_dboptions_t* opt, rocks_wal_filter_t* filter);
  */

  void rocks_dboptions_set_fail_if_options_file_error(rocks_dboptions_t* opt, unsigned char v);

  void rocks_dboptions_set_dump_malloc_stats(rocks_dboptions_t* opt, unsigned char v);

  void rocks_dboptions_set_avoid_flush_during_recovery(rocks_dboptions_t* opt, unsigned char v);

  void rocks_dboptions_set_avoid_flush_during_shutdown(rocks_dboptions_t* opt, unsigned char v);

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

  void rocks_readoptions_destroy(rocks_readoptions_t* opt);

  void rocks_readoptions_set_verify_checksums(
                                              rocks_readoptions_t* opt,
                                              unsigned char v);

  void rocks_readoptions_set_fill_cache(
                                        rocks_readoptions_t* opt, unsigned char v);

  void rocks_readoptions_set_snapshot(
                                      rocks_readoptions_t* opt,
                                      const rocks_snapshot_t* snap);

  void rocks_readoptions_set_iterate_upper_bound(
                                                 rocks_readoptions_t* opt,
                                                 const char* key, size_t keylen);

  void rocks_readoptions_set_read_tier(
                                       rocks_readoptions_t* opt, int v);

  void rocks_readoptions_set_tailing(
                                     rocks_readoptions_t* opt, unsigned char v);

  void rocks_readoptions_set_managed(
                                     rocks_readoptions_t* opt, unsigned char v);


  void rocks_readoptions_set_readahead_size(
                                            rocks_readoptions_t* opt, size_t v);

  void rocks_readoptions_set_pin_data(rocks_readoptions_t* opt,
                                      unsigned char v);

  void rocks_readoptions_set_total_order_seek(rocks_readoptions_t* opt,
                                              unsigned char v);

  void rocks_readoptions_set_prefix_same_as_start(rocks_readoptions_t* opt,
                                                  unsigned char v);

  void rocks_readoptions_set_ignore_range_deletions(rocks_readoptions_t* opt,
                                                    unsigned char v);

  void rocks_readoptions_set_background_purge_on_iterator_cleanup(rocks_readoptions_t* opt,
                                                                  unsigned char v);

  /* > writeoptions */
  rocks_writeoptions_t* rocks_writeoptions_create();

  void rocks_writeoptions_destroy(rocks_writeoptions_t* opt);

  void rocks_writeoptions_set_sync(
                                   rocks_writeoptions_t* opt, unsigned char v);

  void rocks_writeoptions_set_disable_wal(rocks_writeoptions_t* opt, unsigned char v);

  void rocks_writeoptions_set_ignore_missing_column_families(rocks_writeoptions_t* opt, unsigned char v);

  void rocks_writeoptions_set_no_slowdown(rocks_writeoptions_t* opt, unsigned char v);

  /* > compactrange_options */
  rocks_compactrange_options_t* rocks_compactrange_options_create();

  void rocks_compactrange_options_destroy(rocks_compactrange_options_t* opt);

  void rocks_compactrange_options_set_exclusive_manual_compaction(
                                                                  rocks_compactrange_options_t* opt, unsigned char v);

  void rocks_compactrange_options_set_change_level(rocks_compactrange_options_t* opt, unsigned char v);

  void rocks_compactrange_options_set_target_level(rocks_compactrange_options_t* opt, int32_t v);

  void rocks_compactrange_options_set_target_path_id(rocks_compactrange_options_t* opt, uint32_t v);

  void rocks_compactrange_options_set_bottommost_level_compaction(rocks_compactrange_options_t* opt, int v);


  /* > ingestexternalfile_options */
  rocks_ingestexternalfile_options_t* rocks_ingestexternalfile_options_create();

  void rocks_ingestexternalfile_options_destroy(rocks_ingestexternalfile_options_t* opt);

  void rocks_ingestexternalfile_options_set_move_files(rocks_ingestexternalfile_options_t* opt, unsigned char v);
  void rocks_ingestexternalfile_options_set_snapshot_consistency(rocks_ingestexternalfile_options_t* opt, unsigned char v);
  void rocks_ingestexternalfile_options_set_allow_global_seqno(rocks_ingestexternalfile_options_t* opt, unsigned char v);
  void rocks_ingestexternalfile_options_set_allow_blocking_flush(rocks_ingestexternalfile_options_t* opt, unsigned char v);

  /* > misc */
  rocks_logger_t *rocks_create_logger_from_options(const char *path,
                                                   rocks_options_t *opts,
                                                   rocks_status_t *status);


  /* db.h */

  /* > rocks_column_family_handle_t */
  const char* rocks_column_family_handle_get_name(const rocks_column_family_handle_t* handle);

  uint32_t rocks_column_family_handle_get_id(const rocks_column_family_handle_t* handle);

  /* > rocks_db_t */
  rocks_db_t* rocks_db_open(
                            const rocks_options_t* options,
                            const char* name,
                            rocks_status_t* status);

  rocks_db_t* rocks_db_open_for_read_only(
                                          const rocks_options_t* options,
                                          const char* name,
                                          unsigned char error_if_log_file_exist,
                                          rocks_status_t* status);

  void rocks_db_close(rocks_db_t* db);

  rocks_db_t* rocks_db_open_column_families(
                                            const rocks_options_t* db_options,
                                            const char* name,
                                            int num_column_families,
                                            const char* const* column_family_names,
                                            const rocks_cfoptions_t* const* column_family_options,
                                            rocks_column_family_handle_t** column_family_handles,
                                            rocks_status_t *status);

  rocks_db_t* rocks_db_open_for_read_only_column_families(
                                                          const rocks_options_t* db_options,
                                                          const char* name,
                                                          int num_column_families,
                                                          const char** column_family_names,
                                                          const rocks_cfoptions_t** column_family_options,
                                                          rocks_column_family_handle_t** column_family_handles,
                                                          unsigned char error_if_log_file_exist,
                                                          rocks_status_t *status);

  char** rocks_db_list_column_families(
                                       const rocks_options_t* options,
                                       const char* name,
                                       size_t* lencfs,
                                       rocks_status_t* status);

  void rocks_db_list_column_families_destroy(char** list, size_t len);

  rocks_column_family_handle_t* rocks_db_create_column_family(
                                                                 rocks_db_t* db,
                                                                 const rocks_cfoptions_t* column_family_options,
                                                                 const char* column_family_name,
                                                                 rocks_status_t* status);

  rocks_column_family_handle_t* rocks_db_default_column_family(rocks_db_t* db);

  void rocks_db_drop_column_family(
                                   rocks_db_t* db,
                                   rocks_column_family_handle_t* handle,
                                   rocks_status_t* status);

  /* FIXME: when to use? */
  void rocks_db_destroy_column_family_handle(rocks_db_t* db,
                                          rocks_column_family_handle_t* handle,
                                          rocks_status_t* status);

  void rocks_column_family_handle_destroy(rocks_column_family_handle_t* handle);

  void rocks_db_put(
                    rocks_db_t* db,
                    const rocks_writeoptions_t* options,
                    const char* key, size_t keylen,
                    const char* val, size_t vallen,
                    rocks_status_t* status);

  /*
  void rocks_db_put_slice(
                        rocks_db_t* db,
                        const rocks_writeoptions_t* options,
                        const Slice* key, const Slice* value,
                        rocks_status_t* status);
  */
  void rocks_db_put_cf(
                       rocks_db_t* db,
                       const rocks_writeoptions_t* options,
                       rocks_column_family_handle_t* column_family,
                       const char* key, size_t keylen,
                       const char* val, size_t vallen,
                       rocks_status_t* status);

  void rocks_db_delete(
                       rocks_db_t* db,
                       const rocks_writeoptions_t* options,
                       const char* key, size_t keylen,
                       rocks_status_t* status);

  void rocks_db_delete_cf(
                          rocks_db_t* db,
                          const rocks_writeoptions_t* options,
                          rocks_column_family_handle_t* column_family,
                          const char* key, size_t keylen,
                          rocks_status_t* status);

  void rocks_db_single_delete(
                              rocks_db_t* db,
                              const rocks_writeoptions_t* options,
                              const char* key, size_t keylen,
                              rocks_status_t* status);

  void rocks_db_single_delete_cf(
                                 rocks_db_t* db,
                                 const rocks_writeoptions_t* options,
                                 rocks_column_family_handle_t* column_family,
                                 const char* key, size_t keylen,
                                 rocks_status_t* status);

  void rocks_db_delete_range_cf(
                                rocks_db_t* db,
                                const rocks_writeoptions_t* options,
                                rocks_column_family_handle_t* column_family,
                                const char* begin_key, size_t begin_keylen,
                                const char* end_key, size_t end_keylen,
                                rocks_status_t* status);

  void rocks_db_merge(
                      rocks_db_t* db,
                      const rocks_writeoptions_t* options,
                      const char* key, size_t keylen,
                      const char* val, size_t vallen,
                      rocks_status_t* status);

  void rocks_db_merge_cf(
                         rocks_db_t* db,
                         const rocks_writeoptions_t* options,
                         rocks_column_family_handle_t* column_family,
                         const char* key, size_t keylen,
                         const char* val, size_t vallen,
                         rocks_status_t* status);

  void rocks_db_write(
                      rocks_db_t* db,
                      const rocks_writeoptions_t* options,
                      rocks_writebatch_t* batch,
                      rocks_status_t* status);

  char* rocks_db_get(
                     rocks_db_t* db,
                     const rocks_readoptions_t* options,
                     const char* key, size_t keylen,
                     size_t* vallen,
                     rocks_status_t* status);

  char* rocks_db_get_cf(
                        rocks_db_t* db,
                        const rocks_readoptions_t* options,
                        rocks_column_family_handle_t* column_family,
                        const char* key, size_t keylen,
                        size_t* vallen,
                        rocks_status_t* status);

  void rocks_db_multi_get(
    rocks_db_t* db,
    const rocks_readoptions_t* options,
    size_t num_keys, const char* const* keys_list,
    const size_t* keys_list_sizes,
    char** values_list, size_t* values_list_sizes,
    rocks_status_t* status);

  void rocks_db_multi_get_cf(
    rocks_db_t* db,
    const rocks_readoptions_t* options,
    const rocks_column_family_handle_t* const* column_families,
    size_t num_keys, const char* const* keys_list,
    const size_t* keys_list_sizes,
    char** values_list, size_t* values_list_sizes,
    rocks_status_t* status);

  unsigned char rocks_db_key_may_exist(rocks_db_t* db, const rocks_readoptions_t* options,
                                       const char* key, size_t key_len, char** value,
                                       size_t* value_len, unsigned char* value_found);

  unsigned char rocks_db_key_may_exist_cf(rocks_db_t* db, const rocks_readoptions_t* options,
                                          const rocks_column_family_handle_t* column_family,
                                          const char* key, size_t key_len, char* value,
                                          size_t* value_len, unsigned char* value_found);

  rocks_iterator_t* rocks_db_create_iterator(rocks_db_t* db,
                                             const rocks_readoptions_t* options);

  rocks_iterator_t* rocks_db_create_iterator_cf(
                                                rocks_db_t* db,
                                                const rocks_readoptions_t* options,
                                                rocks_column_family_handle_t* column_family);

  void rocks_db_create_iterators(
                                 rocks_db_t *db,
                                 rocks_readoptions_t* opts,
                                 rocks_column_family_handle_t** column_families,
                                 rocks_iterator_t** iterators,
                                 size_t size,
                                 rocks_status_t* status);


  rocks_snapshot_t* rocks_db_get_snapshot(rocks_db_t* db);

  void rocks_db_release_snapshot(rocks_db_t* db, rocks_snapshot_t* snapshot);

  void rocks_db_compact_range(
                              rocks_db_t* db,
                              const char* start_key, size_t start_key_len,
                              const char* limit_key, size_t limit_key_len);

  void rocks_db_compact_range_cf(
                                rocks_db_t* db,
                                rocks_column_family_handle_t* column_family,
                                const char* start_key, size_t start_key_len,
                                const char* limit_key, size_t limit_key_len);

  void rocks_db_compact_range_opt(rocks_db_t* db, rocks_compactrange_options_t* opt,
                                  const char* start_key, size_t start_key_len,
                                  const char* limit_key, size_t limit_key_len,
                                  rocks_status_t *status);
   
  void rocks_db_compact_range_cf_opt(rocks_db_t* db,
                                     rocks_column_family_handle_t* column_family,
                                     rocks_compactrange_options_t* opt,
                                     const char* start_key, size_t start_key_len,
                                     const char* limit_key, size_t limit_key_len);

  const char* rocks_db_get_name(rocks_db_t* db, size_t* len);
  
  void rocks_db_ingest_external_file(rocks_db_t* db,
                                     const char* const* file_list,
                                     const size_t* file_list_sizes,
                                     size_t file_len,
                                     const rocks_ingestexternalfile_options_t* options,
                                     rocks_status_t* status);
    
  void rocks_db_ingest_external_file_cf(rocks_db_t* db,
                                        rocks_column_family_handle_t* column_family,
                                        const char* const* file_list,
                                        const size_t* file_list_sizes,
                                        size_t file_len,
                                        const rocks_ingestexternalfile_options_t* options,
                                        rocks_status_t* status);
  
  /*    pub fn */
  void rocks_destroy_db(
                        const rocks_options_t* options,
                        const char* name,
                        rocks_status_t* status);

  void rocks_repair_db(
                       const rocks_options_t* options,
                       const char* name,
                       rocks_status_t* status);

  /* rate_limiter.h */
  rocks_ratelimiter_t* rocks_ratelimiter_create(
                                                int64_t rate_bytes_per_sec,
                                                int64_t refill_period_us,
                                                int32_t fairness);

  void rocks_ratelimiter_destroy(rocks_ratelimiter_t *limiter);


  /* env.h */
  rocks_env_t* rocks_create_default_env();

  rocks_env_t* rocks_create_mem_env();

  void rocks_env_set_background_threads(rocks_env_t* env, int n);

  void rocks_env_set_high_priority_background_threads(rocks_env_t* env, int n);

  void rocks_env_join_all_threads(rocks_env_t* env);

  void rocks_env_destroy(rocks_env_t* env);

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


  void rocks_logger_destroy(rocks_logger_t *logger);

  /* snapshot.h */

  const rocks_snapshot_t* rocks_create_snapshot(rocks_db_t* db);

  void rocks_release_snapshot(
                              rocks_db_t* db,
                              const rocks_snapshot_t* snapshot);

  /* iterator.h */

  /* write_batch.h */

  rocks_writebatch_t* rocks_writebatch_create();

  rocks_writebatch_t* rocks_writebatch_create_with_reserved_bytes(size_t size);

  void rocks_writebatch_destroy(rocks_writebatch_t* b);

  void rocks_writebatch_clear(rocks_writebatch_t* b);

  int rocks_writebatch_count(rocks_writebatch_t* b);

  void rocks_writebatch_put(
                            rocks_writebatch_t* b,
                            const char* key, size_t klen,
                            const char* val, size_t vlen);

  void rocks_writebatch_put_cf(
                               rocks_writebatch_t* b,
                               rocks_column_family_handle_t* column_family,
                               const char* key, size_t klen,
                               const char* val, size_t vlen);

  void rocks_writebatch_putv(
                             rocks_writebatch_t* b,
                             int num_keys, const char* const* keys_list,
                             const size_t* keys_list_sizes,
                             int num_values, const char* const* values_list,
                             const size_t* values_list_sizes);

  void rocks_writebatch_putv_cf(
                                rocks_writebatch_t* b,
                                rocks_column_family_handle_t* column_family,
                                int num_keys, const char* const* keys_list,
                                const size_t* keys_list_sizes,
                                int num_values, const char* const* values_list,
                                const size_t* values_list_sizes);

  void rocks_writebatch_merge(
                              rocks_writebatch_t* b,
                              const char* key, size_t klen,
                              const char* val, size_t vlen);

  void rocks_writebatch_merge_cf(
                                 rocks_writebatch_t* b,
                                 rocks_column_family_handle_t* column_family,
                                 const char* key, size_t klen,
                                 const char* val, size_t vlen);

  void rocks_writebatch_mergev(
                               rocks_writebatch_t* b,
                               int num_keys, const char* const* keys_list,
                               const size_t* keys_list_sizes,
                               int num_values, const char* const* values_list,
                               const size_t* values_list_sizes);

  void rocks_writebatch_mergev_cf(
                                  rocks_writebatch_t* b,
                                  rocks_column_family_handle_t* column_family,
                                  int num_keys, const char* const* keys_list,
                                  const size_t* keys_list_sizes,
                                  int num_values, const char* const* values_list,
                                  const size_t* values_list_sizes);

  void rocks_writebatch_delete(
                               rocks_writebatch_t* b,
                               const char* key, size_t klen);

  void rocks_writebatch_delete_cf(
                                  rocks_writebatch_t* b,
                                  rocks_column_family_handle_t* column_family,
                                  const char* key, size_t klen);

  void rocks_writebatch_deletev(
                                rocks_writebatch_t* b,
                                int num_keys, const char* const* keys_list,
                                const size_t* keys_list_sizes);

  void rocks_writebatch_deletev_cf(
                                   rocks_writebatch_t* b,
                                   rocks_column_family_handle_t* column_family,
                                   int num_keys, const char* const* keys_list,
                                   const size_t* keys_list_sizes);

  void rocks_writebatch_single_delete(
                                      rocks_writebatch_t* b,
                                      const char* key, size_t klen);

  void rocks_writebatch_single_delete_cf(
                                         rocks_writebatch_t* b,
                                         rocks_column_family_handle_t* column_family,
                                         const char* key, size_t klen);

  void rocks_writebatch_delete_range(rocks_writebatch_t* b,
                                     const char* start_key,
                                     size_t start_key_len, const char* end_key,
                                     size_t end_key_len);

  void rocks_writebatch_delete_range_cf(
                                        rocks_writebatch_t* b, rocks_column_family_handle_t* column_family,
                                        const char* start_key, size_t start_key_len, const char* end_key,
                                        size_t end_key_len);

  void rocks_writebatch_delete_rangev(rocks_writebatch_t* b, int num_keys,
                                      const char* const* start_keys_list,
                                      const size_t* start_keys_list_sizes,
                                      const char* const* end_keys_list,
                                      const size_t* end_keys_list_sizes);

  void rocks_writebatch_delete_rangev_cf(
                                         rocks_writebatch_t* b, rocks_column_family_handle_t* column_family,
                                         int num_keys, const char* const* start_keys_list,
                                         const size_t* start_keys_list_sizes, const char* const* end_keys_list,
                                         const size_t* end_keys_list_sizes);

  void rocks_writebatch_put_log_data(
                                     rocks_writebatch_t* b,
                                     const char* blob, size_t len);
  void rocks_writebatch_iterate(
                                rocks_writebatch_t* b,
                                void* state,
                                void (*put)(void*, const char* k, size_t klen, const char* v, size_t vlen),
                                void (*deleted)(void*, const char* k, size_t klen));

  const char* rocks_writebatch_data(rocks_writebatch_t* b, size_t* size);

  void rocks_writebatch_set_save_point(rocks_writebatch_t* b);

  void rocks_writebatch_rollback_to_save_point(rocks_writebatch_t* b,
                                               rocks_status_t* status);

  /* iterator */
  void rocks_iter_destroy(rocks_iterator_t* iter);

  unsigned char rocks_iter_valid(const rocks_iterator_t* iter);

  void rocks_iter_seek_to_first(rocks_iterator_t* iter);

  void rocks_iter_seek_to_last(rocks_iterator_t* iter);

  void rocks_iter_seek(rocks_iterator_t* iter, const char* k, size_t klen);

  void rocks_iter_seek_for_prev(rocks_iterator_t* iter, const char* k,
                                size_t klen);

  void rocks_iter_next(rocks_iterator_t* iter);

  void rocks_iter_prev(rocks_iterator_t* iter);

  const char* rocks_iter_key(const rocks_iterator_t* iter, size_t* klen);

  const char* rocks_iter_value(const rocks_iterator_t* iter, size_t* vlen);

  void rocks_iter_get_status(const rocks_iterator_t* iter, rocks_status_t* status);

  /* cache */
  rocks_cache_t* rocks_cache_create_lru(size_t capacity,
                                        int num_shard_bits,
                                        char strict_capacity_limit,
                                        double high_pri_pool_ratio);

  rocks_cache_t* rocks_cache_create_clock(size_t capacity, int num_shard_bits, char strict_capacity_limit);

  void rocks_cache_destroy(rocks_cache_t* cache);

  void rocks_cache_set_capacity(rocks_cache_t* cache, size_t capacity);

  size_t rocks_cache_get_capacity(rocks_cache_t* cache);

  size_t rocks_cache_get_usage(rocks_cache_t* cache);

  size_t rocks_cache_get_pinned_usage(rocks_cache_t* cache);

  const char* rocks_cache_name(rocks_cache_t* cache);

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
                                                        const rocks_envoptions_t* env_options,
                                                        const rocks_options_t* options,
                                                        const rocks_c_comparator_t* comparator, /* avoid export rocksdb::Comparator type */
                                                        rocks_column_family_handle_t* column_family,
                                                        unsigned char invalidate_page_cache);

  rocks_sst_file_writer_t* rocks_sst_file_writer_create_from_rust_comparator(
                                                                             const rocks_envoptions_t* env_options,
                                                                             const rocks_options_t* options,
                                                                             const void* comparator,
                                                                             rocks_column_family_handle_t* column_family,
                                                                             unsigned char invalidate_page_cache);

  void rocks_sst_file_writer_destroy(rocks_sst_file_writer_t* writer);

  void rocks_sst_file_writer_open(rocks_sst_file_writer_t* writer,
                                  const char* file_path,
                                  const size_t file_path_len,
                                  rocks_status_t *status);

  void rocks_sst_file_writer_add(rocks_sst_file_writer_t* writer,
                                 const char* key, const size_t key_len,
                                 const char* value, const size_t value_len,
                                 rocks_status_t *status);

  void rocks_sst_file_writer_finish(rocks_sst_file_writer_t* writer,
                                    rocks_external_sst_file_info_t* info,
                                    rocks_status_t *status);

  uint64_t rocks_sst_file_writer_file_size(rocks_sst_file_writer_t* writer);

/* comparator */
  /* avoid export rocksdb::Comparator type */
  const rocks_c_comparator_t* rocks_comparator_bytewise();
  const rocks_c_comparator_t* rocks_comparator_bytewise_reversed();


  /* version */
  int rocks_version_major();
  int rocks_version_minor();
  int rocks_version_patch();

  /* aux */
  void free(void *p);

  size_t cxx_vector_slice_size(const void* list);
  const void* cxx_vector_slice_nth(const void* list, size_t n);

  void cxx_string_assign(void* s, const char* p, size_t len);
  const char* cxx_string_data(const void *s);
  size_t cxx_string_size(const void *s);


#ifdef __cplusplus
}
#endif
