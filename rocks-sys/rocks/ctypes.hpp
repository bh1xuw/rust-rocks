#pragma once

#ifndef __RUST_ROCSK_SYS_H____
#define __RUST_ROCSK_SYS_H____



#include "rocksdb/status.h"
#include "rocksdb/db.h"
#include "rocksdb/options.h"
#include "rocksdb/rate_limiter.h"
#include "rocksdb/env.h"
#include "rocksdb/iterator.h"
#include "rocksdb/cache.h"
#include "rocksdb/merge_operator.h"
#include "rocksdb/sst_file_writer.h"

#include "rust_export.h"

#include <iostream>

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
  struct rocks_dbpath_t                { DbPath                        rep; };
  struct rocks_dboptions_t             { DBOptions                     rep; };
  struct rocks_cfoptions_t             { ColumnFamilyOptions           rep; };
  struct rocks_options_t               { Options                       rep; };
  struct rocks_readoptions_t           {
    ReadOptions rep;
    Slice       upper_bound; // stack variable to set pointer to in ReadOptions
  };
  struct rocks_writeoptions_t              { WriteOptions              rep; };
  struct rocks_flushoptions_t              { FlushOptions              rep; };
  struct rocks_compaction_options_t        { CompactionOptions         rep; };
  struct rocks_compactrange_options_t      { CompactRangeOptions       rep; };
  struct rocks_ingestexternalfile_options_t { IngestExternalFileOptions rep; };

  struct rocks_mergeoperator_t: public MergeOperator {
    void* obj;                  // rust Box<trait obj>

    rocks_mergeoperator_t(void *trait_obj): obj(trait_obj) {}

    ~rocks_mergeoperator_t() {
      rust_merge_operator_drop(this->obj);
    }

    const char* Name() const override {
      return rust_merge_operator_name(this->obj);
    }

    virtual bool FullMergeV2(const MergeOperationInput& merge_in,
                             MergeOperationOutput* merge_out) const override {
      auto ret = rust_merge_operator_call_full_merge_v2(
                                                    this->obj,
                                                    &merge_in,
                                                    merge_out);

      if (merge_out->existing_operand.data() != nullptr) {
        merge_out->new_value.clear();
      }
      return ret != 0;
    }
  };

  struct rocks_associative_mergeoperator_t: public AssociativeMergeOperator {
    void* obj;                  // rust Box<trait obj>

    rocks_associative_mergeoperator_t(void *trait_obj): obj(trait_obj) {}

    ~rocks_associative_mergeoperator_t() {
      rust_associative_merge_operator_drop(this->obj);
    }

    const char* Name() const override {
      return rust_associative_merge_operator_name(this->obj);
    }

    bool Merge(const Slice& key,
                       const Slice* existing_value,
                       const Slice& value,
                       std::string* new_value,
                       Logger* logger) const override {
      char* nval = nullptr;
      size_t nval_len = 0;
      auto ret = rust_associative_merge_operator_call(this->obj,
                                                      &key,
                                                      existing_value,
                                                      &value,
                                                      &nval, &nval_len,
                                                      logger);
      if (ret) {
        new_value->assign(nval, nval_len);
        // NOTE: this drops Vec<u8>
        rust_drop_vec_u8(nval, nval_len);
      }
      return (bool)ret;
    }
  };

  /* comparator */
  struct rocks_comparator_t: public Comparator {
    void* obj;                  // rust Box<trait obj>
    
    rocks_comparator_t(void *trait_obj): obj(trait_obj) {}

    ~rocks_comparator_t() {
      rust_comparator_drop(this->obj);
    }

    int Compare(const Slice& a, const Slice& b) const override {
      return rust_comparator_compare(this->obj,
                                     &a, &b);
    }

    bool Equal(const Slice& a, const Slice& b) const override {
      return rust_comparator_equal(this->obj,
                                   &a, &b);
    }

    const char* Name() const override {
      return rust_comparator_name(this->obj);
    }

    void FindShortestSeparator(
                               std::string* start,
                               const Slice& limit) const override {
      rust_comparator_find_shortest_separator(this->obj,
                                              start, &limit);
    }

    void FindShortSuccessor(std::string* key) const override {
      rust_comparator_find_short_successor(this->obj, key);
    }
  };

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

  /* iterator */
  struct rocks_iterator_t        { Iterator*         rep; };

  /* write_batch */
  struct rocks_writebatch_t      { WriteBatch        rep; };

  /* cache */
  struct rocks_cache_t           { shared_ptr<Cache>   rep; };

  /* sst_file_writer */
  struct rocks_sst_file_writer_t { SstFileWriter* rep; };

  /* aux */
  static bool SaveError(rocks_status_t* status, const Status& s) {
    assert(status != nullptr);
    rocks_status_convert(&s, status);
    return !s.ok();
  }

  static char* CopyString(const std::string& str) {
    char* result = reinterpret_cast<char*>(malloc(sizeof(char) * str.size()));
    memcpy(result, str.data(), sizeof(char) * str.size());
    return result;
  }


#ifdef __cplusplus
}
#endif

#endif /* __RUST_ROCSK_SYS_H____ */

