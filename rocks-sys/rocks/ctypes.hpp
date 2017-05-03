#pragma once

#include "rocksdb/status.h"
#include "rocksdb/db.h"
#include "rocksdb/options.h"
#include "rocksdb/rate_limiter.h"
#include "rocksdb/env.h"

using namespace rocksdb;

#ifdef __cplusplus
extern "C" {
#endif
#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>

  /* status */
  typedef struct rocks_status_t {
    int code;
    int sub_code;
    const char *state;
  } rocks_status_t;

  void rocks_status_convert(const Status *status, rocks_status_t *p);

  /* db */

  struct rocks_column_family_descriptor_t { DB*                 rep; };
  struct rocks_column_family_handle_t     { ColumnFamilyHandle* rep; };
  struct rocks_db_t                       { DB*                 rep; };

  /* options */ 
  struct rocks_column_family_options_t       { ColumnFamilyOptions       rep; };
  struct rocks_dbpath_t                    { DbPath                    rep; };
  struct rocks_db_options_t                 { DBOptions                 rep; };
  struct rocks_options_t                   { Options                   rep; };
  struct rocks_readoptions_t               {
    ReadOptions rep;
    Slice upper_bound; // stack variable to set pointer to in ReadOptions
  };
  struct rocks_writeoptions_t              { WriteOptions              rep; };
  struct rocks_flushoptions_t              { FlushOptions              rep; };
  struct rocks_compaction_options_t        { CompactionOptions         rep; };
  struct rocks_compactrange_options_t      { CompactRangeOptions       rep; };
  struct rocks_ingestexternalfileoptions_t { IngestExternalFileOptions rep; };

  /* rate_limiter */

  struct rocks_ratelimiter_t { RateLimiter* rep; };

  /* env */
  struct rocks_envoptions_t      { EnvOptions        rep; };
  struct rocks_logger_t          { shared_ptr<Logger>  rep; };

  struct rocks_env_t {
    Env* rep;
    bool is_default;
  };

  /* snapshot*/
  struct rocks_snapshot_t        { const Snapshot*   rep; };




  /* aux */
  static bool SaveError(rocks_status_t* status, const Status& s) {
    assert(status != nullptr);
    rocks_status_convert(&s, status);
    return s.ok();
  }

  static char* CopyString(const std::string& str) {
    char* result = reinterpret_cast<char*>(malloc(sizeof(char) * str.size()));
    memcpy(result, str.data(), sizeof(char) * str.size());
    return result;
  }


#ifdef __cplusplus
}
#endif
