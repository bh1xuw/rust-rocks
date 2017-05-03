#pragma once

#ifdef __cplusplus
extern "C" {
#endif
#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>

  /* db.h */

  typedef struct rocks_column_family_descriptor_t rocks_column_family_descriptor_t ;
  typedef struct rocks_column_family_handle_t     rocks_column_family_handle_t     ;
  typedef struct rocks_db_t                       rocks_db_t                       ;

  /* options.h */
  typedef struct rocks_column_family_options_t       rocks_column_family_options_t;
  typedef struct rocks_dbpath_t                    rocks_dbpath_t;
  typedef struct rocks_dboptions_t                 rocks_dboptions_t;
  typedef struct rocks_options_t                   rocks_options_t ;
  typedef struct rocks_readoptions_t               rocks_readoptions_t;
  typedef struct rocks_writeoptions_t              rocks_writeoptions_t;
  typedef struct rocks_flushoptions_t              rocks_flushoptions_t;
  typedef struct rocks_compactionoptions_t         rocks_compactionoptions_t;
  typedef struct rocks_compactrangeoptions_t       rocks_compactrangeoptions_t;
  typedef struct rocks_ingestexternalfileoptions_t rocks_ingestexternalfileoptions_t;

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


  /* ****************************** functions ****************************** */
  /* options.h */
  rocks_options_t* rocks_options_create();
  void rocks_options_destroy(rocks_options_t* options);

  void rocks_options_set_create_if_missing(rocks_options_t* opt, unsigned char v);

  /* column_family_options */
  rocks_column_family_options_t* rocks_column_family_options_create();

  void rocks_column_family_options_destroy(rocks_column_family_options_t* options);

  /* member functions */
  void rocks_options_increase_parallelism(rocks_options_t* opt, int total_threads);
  void rocks_options_optimize_for_point_lookup(
                                               rocks_options_t* opt, uint64_t block_cache_size_mb);

  void rocks_options_optimize_level_style_compaction(
                                                     rocks_options_t* opt, uint64_t memtable_memory_budget);

  void rocks_options_optimize_universal_style_compaction(
                                                         rocks_options_t* opt, uint64_t memtable_memory_budget);

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

  void rocks_readoptions_set_readahead_size(
                                            rocks_readoptions_t* opt, size_t v);

  void rocks_readoptions_set_pin_data(rocks_readoptions_t* opt,
                                      unsigned char v);

  void rocks_readoptions_set_total_order_seek(rocks_readoptions_t* opt,
                                              unsigned char v);

  /* writeoptions */
  rocks_writeoptions_t* rocks_writeoptions_create();

  void rocks_writeoptions_destroy(rocks_writeoptions_t* opt);

  void rocks_writeoptions_set_sync(
                                   rocks_writeoptions_t* opt, unsigned char v);

  void rocks_writeoptions_disable_WAL(rocks_writeoptions_t* opt, int disable);

  rocks_logger_t *rocks_create_logger_from_options(const char *path,
                                                   rocks_options_t *opts,
                                                   rocks_status_t *status);




  /* db.h */
  rocks_db_t* rocks_db_open(
                            const rocks_options_t* options,
                            const char* name,
                            rocks_status_t* status);

  void rocks_db_close(rocks_db_t* db);

  rocks_db_t* rocks_db_open_for_read_only(
                                          const rocks_options_t* options,
                                          const char* name,
                                          unsigned char error_if_log_file_exist,
                                          rocks_status_t* status);

  char** rocks_db_list_column_families(
                                       const rocks_options_t* options,
                                       const char* name,
                                       size_t* lencfs,
                                       rocks_status_t* status);

  void rocks_db_list_column_families_destroy(char** list, size_t len);

  rocks_column_family_handle_t* rocks_db_create_column_family(
                                                                 rocks_db_t* db,
                                                                 const rocks_options_t* column_family_options,
                                                                 const char* column_family_name,
                                                                 rocks_status_t* status);

  void rocks_db_drop_column_family(
                                   rocks_db_t* db,
                                   rocks_column_family_handle_t* handle,
                                   rocks_status_t* status);

  void rocks_db_column_family_handle_destroy(rocks_column_family_handle_t* handle);




  void rocks_db_put(
                    rocks_db_t* db,
                    const rocks_writeoptions_t* options,
                    const char* key, size_t keylen,
                    const char* val, size_t vallen,
                    rocks_status_t* status);


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

  void rocks_logger_destroy(rocks_logger_t *logger);

  /* snapshot.h */

  const rocks_snapshot_t* rocks_create_snapshot(rocks_db_t* db);

  void rocks_release_snapshot(
                              rocks_db_t* db,
                              const rocks_snapshot_t* snapshot);



#ifdef __cplusplus
}
#endif
