#pragma once

#ifndef __RUST_ROCSK_SYS_H____
#define __RUST_ROCSK_SYS_H____

#include "rocksdb/cache.h"
#include "rocksdb/compaction_filter.h"
#include "rocksdb/db.h"
#include "rocksdb/db_dump_tool.h"
#include "rocksdb/env.h"
#include "rocksdb/filter_policy.h"
#include "rocksdb/iterator.h"
#include "rocksdb/merge_operator.h"
#include "rocksdb/metadata.h"
#include "rocksdb/options.h"
#include "rocksdb/rate_limiter.h"
#include "rocksdb/slice_transform.h"
#include "rocksdb/sst_file_writer.h"
#include "rocksdb/status.h"
#include "rocksdb/table.h"
#include "rocksdb/table_properties.h"
#include "rocksdb/transaction_log.h"
#include "rocksdb/utilities/debug.h"
#include "rocksdb/write_buffer_manager.h"

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
struct rocks_status_t {
  Status rep;

  // rocks_status_t(const Status st) noexcept : rep(st) {}
  rocks_status_t() : rep(Status()) {}
  rocks_status_t(const Status&& st) noexcept : rep(std::move(st)) {}
};
/* slice */
struct rocks_pinnable_slice_t {
  PinnableSlice rep;
};

/* db */
struct rocks_column_family_descriptor_t {
  DB* rep;
};
struct rocks_column_family_handle_t {
  ColumnFamilyHandle* rep;
};
struct rocks_db_t {
  DB* rep;
};

/* options */
struct rocks_dbpath_t {
  DbPath rep;
};
struct rocks_dboptions_t {
  DBOptions rep;
};
struct rocks_cfoptions_t {
  ColumnFamilyOptions rep;
};
struct rocks_options_t {
  Options rep;
};
struct rocks_readoptions_t {
  ReadOptions rep;
  Slice upper_bound;  // stack variable to set pointer to in ReadOptions
};
struct rocks_writeoptions_t {
  WriteOptions rep;
};
struct rocks_flushoptions_t {
  FlushOptions rep;
};
struct rocks_fifo_compaction_options_t {
  CompactionOptionsFIFO rep;
};

struct rocks_compaction_options_t {
  CompactionOptions rep;
};
struct rocks_compactrange_options_t {
  CompactRangeOptions rep;
};
struct rocks_ingestexternalfile_options_t {
  IngestExternalFileOptions rep;
};

struct rocks_mergeoperator_t : public MergeOperator {
  void* obj;  // rust Box<trait obj>

  rocks_mergeoperator_t(void* trait_obj) : obj(trait_obj) {}

  ~rocks_mergeoperator_t() { rust_merge_operator_drop(this->obj); }

  const char* Name() const override {
    return rust_merge_operator_name(this->obj);
  }

  virtual bool FullMergeV2(const MergeOperationInput& merge_in,
                           MergeOperationOutput* merge_out) const override {
    auto ret =
        rust_merge_operator_call_full_merge_v2(this->obj, &merge_in, merge_out);

    if (merge_out->existing_operand.data() != nullptr) {
      merge_out->new_value.clear();
    }
    return ret != 0;
  }
};

struct rocks_associative_mergeoperator_t : public AssociativeMergeOperator {
  void* obj;  // rust Box<trait obj>

  rocks_associative_mergeoperator_t(void* trait_obj) : obj(trait_obj) {}

  ~rocks_associative_mergeoperator_t() {
    rust_associative_merge_operator_drop(this->obj);
  }

  const char* Name() const override {
    return rust_associative_merge_operator_name(this->obj);
  }

  bool Merge(const Slice& key, const Slice* existing_value, const Slice& value,
             std::string* new_value, Logger* logger) const override {
    char* nval = nullptr;
    size_t nval_len = 0;
    auto ret = rust_associative_merge_operator_call(
        this->obj, &key, existing_value, &value, &nval, &nval_len, logger);
    if (ret) {
      new_value->assign(nval, nval_len);
      // NOTE: this drops Vec<u8>
      rust_drop_vec_u8(nval, nval_len);
    }
    return (bool)ret;
  }
};

/* comparator */
struct rocks_comparator_t : public Comparator {
  void* obj;  // rust Box<trait obj>

  rocks_comparator_t(void* trait_obj) : obj(trait_obj) {}

  // FIXME: since Options->comparator is a raw pointer
  //  this may not be called
  ~rocks_comparator_t() { rust_comparator_drop(this->obj); }

  int Compare(const Slice& a, const Slice& b) const override {
    return rust_comparator_compare(this->obj, &a, &b);
  }

  bool Equal(const Slice& a, const Slice& b) const override {
    return rust_comparator_equal(this->obj, &a, &b);
  }

  const char* Name() const override { return rust_comparator_name(this->obj); }

  void FindShortestSeparator(std::string* start,
                             const Slice& limit) const override {
    rust_comparator_find_shortest_separator(this->obj, start, &limit);
  }

  void FindShortSuccessor(std::string* key) const override {
    rust_comparator_find_short_successor(this->obj, key);
  }
};

/* rate_limiter */
struct rocks_ratelimiter_t {
  shared_ptr<RateLimiter> rep;
};

/* env */
struct rocks_envoptions_t {
  EnvOptions rep;
};
struct rocks_logger_t {
  shared_ptr<Logger> rep;
};

struct rocks_env_t {
  Env* rep;
  bool is_default;
};

/* snapshot*/
struct rocks_snapshot_t {
  const Snapshot* rep;
};

/* iterator */
struct rocks_iterator_t {
  Iterator* rep;
};

/* write_batch */
struct rocks_writebatch_t {
  std::unique_ptr<WriteBatch> rep;
};
typedef struct rocks_raw_writebatch_t rocks_raw_writebatch_t;

struct rocks_writebatch_handler_t : public WriteBatch::Handler {
  void* obj;  // rust Box<trait obj>

  rocks_writebatch_handler_t(void* trait_obj) : obj(trait_obj) {}

  ~rocks_writebatch_handler_t() { rust_write_batch_handler_drop(this->obj); }

  Status PutCF(uint32_t column_family_id, const Slice& key,
               const Slice& value) override {
    rust_write_batch_handler_put_cf(this->obj, column_family_id, &key, &value);
    return Status::OK();
  }

  Status DeleteCF(uint32_t column_family_id, const Slice& key) override {
    rust_write_batch_handler_delete_cf(this->obj, column_family_id, &key);
    return Status::OK();
  }

  Status SingleDeleteCF(uint32_t column_family_id, const Slice& key) override {
    rust_write_batch_handler_single_delete_cf(this->obj, column_family_id,
                                              &key);
    return Status::OK();
  }

  Status DeleteRangeCF(uint32_t column_family_id, const Slice& begin_key,
                       const Slice& end_key) override {
    rust_write_batch_handler_delete_range_cf(this->obj, column_family_id,
                                             &begin_key, &end_key);
    return Status::OK();
  }
  Status MergeCF(uint32_t column_family_id, const Slice& key,
                 const Slice& value) override {
    rust_write_batch_handler_merge_cf(this->obj, column_family_id, &key,
                                      &value);
    return Status::OK();
  }

  void LogData(const Slice& blob) override {
    rust_write_batch_handler_log_data(this->obj, &blob);
  }

  Status MarkBeginPrepare() override {
    rust_write_batch_handler_mark_begin_prepare(this->obj);
    return Status::OK();
  }

  Status MarkEndPrepare(const Slice& xid) override {
    rust_write_batch_handler_mark_end_prepare(this->obj, &xid);
    return Status::OK();
  }

  Status MarkRollback(const Slice& xid) override {
    rust_write_batch_handler_mark_rollback(this->obj, &xid);
    return Status::OK();
  }

  Status MarkCommit(const Slice& xid) override {
    rust_write_batch_handler_mark_commit(this->obj, &xid);
    return Status::OK();
  }

  bool Continue() override {
    return rust_write_batch_handler_will_continue(this->obj);
  }
};

/* table */
struct rocks_block_based_table_options_t {
  BlockBasedTableOptions rep;
};
struct rocks_cuckoo_table_options_t {
  CuckooTableOptions rep;
};
struct rocks_plain_table_options_t {
  PlainTableOptions rep;
};

/* filter_policy */
struct rocks_raw_filterpolicy_t {
  shared_ptr<const FilterPolicy> rep;
};

/* cache */
struct rocks_cache_t {
  shared_ptr<Cache> rep;
};

/* sst_file_writer */
struct rocks_sst_file_writer_t {
  SstFileWriter* rep;
};
struct rocks_external_sst_file_info_t {
  ExternalSstFileInfo rep;
};

/* compaction_filter */
struct rocks_compaction_filter_t : public CompactionFilter {
  void* obj;  // rust Box<trait obj>

  rocks_compaction_filter_t(void* trait_obj) : obj(trait_obj) {}

  ~rocks_compaction_filter_t() { rust_compaction_filter_drop(this->obj); }

  Decision FilterV2(int level, const Slice& key, ValueType value_type,
                    const Slice& existing_value, std::string* new_value,
                    std::string* skip_until) const override {
    auto ret =
        rust_compaction_filter_call(this->obj, level, &key, value_type,
                                    &existing_value, new_value, skip_until);
    return static_cast<CompactionFilter::Decision>(ret);
  }

  bool IgnoreSnapshots() const override {
    return rust_compaction_filter_ignore_snapshots(this->obj) != 0;
  }

  const char* Name() const override {
    return rust_compaction_filter_name(this->obj);
  }
};

/* slice_transform */
struct rocks_slice_transform_t : public SliceTransform {
  void* obj;  // rust Box<trait obj>

  rocks_slice_transform_t(void* trait_obj) : obj(trait_obj) {}

  ~rocks_slice_transform_t() { rust_slice_transform_drop(this->obj); }

  const char* Name() const override {
    return rust_slice_transform_name(this->obj);
  }

  Slice Transform(const Slice& key) const override {
    char* ret = nullptr;
    size_t ret_len = 0;
    rust_slice_transform_call(this->obj, &key, &ret, &ret_len);
    return Slice(ret, ret_len);
  }

  bool InDomain(const Slice& key) const override {
    return rust_slice_transform_in_domain(this->obj, &key) != 0;
  }

  // not used and remains here for backward compatibility.
  bool InRange(const Slice& dst) const override { return false; }
};

/* db_dump_tool */
struct rocks_dump_options_t {
  DumpOptions rep;
};
struct rocks_undump_options_t {
  UndumpOptions rep;
};

/* iostats_context */
typedef struct rocks_iostats_context_t rocks_iostats_context_t;

/* perf_context */
typedef struct rocks_perf_context_t rocks_perf_context_t;

/* statistics */
struct rocks_statistics_t {
  shared_ptr<Statistics> rep;
};
typedef struct rocks_histogram_data_t rocks_histogram_data_t;

/* metadata */
struct rocks_livefiles_t {
  std::vector<LiveFileMetaData> rep;
};
struct rocks_column_family_metadata_t {
  ColumnFamilyMetaData rep;
};

/* universal_compaction */
struct rocks_universal_compaction_options_t {
  rocksdb::CompactionOptionsUniversal rep;
};

/* transaction_log */
struct rocks_logfiles_t {
  VectorLogPtr rep;
};

struct rocks_transaction_log_iterator_t {
  std::unique_ptr<TransactionLogIterator> rep;
};

/* table_properties */
struct rocks_table_props_collection_t {
  // std::unordered_map<std::string, std::shared_ptr<const TableProperties>>
  TablePropertiesCollection rep;
};

struct rocks_table_props_collection_iter_t {
  TablePropertiesCollection::const_iterator rep;
  const TablePropertiesCollection::const_iterator cend;
};

struct rocks_table_props_t {
  std::shared_ptr<const TableProperties> rep;
};

// std::map<std::string, std::string>*
// ie. UserCollectedProperties*
typedef struct rocks_user_collected_props_t rocks_user_collected_props_t;

struct rocks_user_collected_props_iter_t {
  UserCollectedProperties::const_iterator rep;
  const UserCollectedProperties::const_iterator cend;
};

struct rocks_table_props_collector_t : public TablePropertiesCollector {
  void* obj;  // rust Box<trait obj>

  rocks_table_props_collector_t(void* trait_obj) : obj(trait_obj) {}

  ~rocks_table_props_collector_t() {
    rust_table_props_collector_drop(this->obj);
  }

  const char* Name() const override {
    return rust_table_props_collector_name(this->obj);
  }

  Status AddUserKey(const Slice& key, const Slice& value, EntryType type,
                    SequenceNumber seq, uint64_t file_size) override {
    rust_table_props_collector_add_user_key(
        this->obj, &key, &value, static_cast<int>(type), seq, file_size);
    return Status::OK();
  }

  Status Finish(UserCollectedProperties* properties) override {
    rust_table_props_collector_finish(this->obj, properties);
    return Status::OK();
  }

  // TODO:
  UserCollectedProperties GetReadableProperties() const override {
    return UserCollectedProperties{};
  }

  bool NeedCompact() const override {
    return rust_table_props_collector_need_compact(this->obj);
  }
};

struct rocks_table_props_collector_factory_t
    : public TablePropertiesCollectorFactory {
  void* obj;  // rust Box<trait obj>

  rocks_table_props_collector_factory_t(void* trait_obj) : obj(trait_obj) {}

  ~rocks_table_props_collector_factory_t() {
    rust_table_props_collector_factory_drop(this->obj);
  }

  const char* Name() const override {
    return rust_table_props_collector_factory_name(this->obj);
  }

  TablePropertiesCollector* CreateTablePropertiesCollector(
      TablePropertiesCollectorFactory::Context context) override {
    auto collector = rust_table_props_collector_factory_new_collector(
        this->obj, context.column_family_id);
    return new rocks_table_props_collector_t(collector);
  }
};

/* write_buffer_manager */
struct rocks_write_buffer_manager_t {
  std::shared_ptr<WriteBufferManager> rep;
};

/* debug */
typedef struct rocks_key_version_t rocks_key_version_t;

struct rocks_key_version_collection_t {
  std::vector<KeyVersion> rep;
};

/* aux */
struct cxx_string_vector_t {
  std::vector<std::string> rep;
};

static bool SaveError(rocks_status_t** status, const Status&& s) {
  if (s.ok()) {
    *status = nullptr;
    return false;
  } else {
    *status = new rocks_status_t{std::move(s)};
    return true;
  }
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
