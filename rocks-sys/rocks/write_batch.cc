#include "rocksdb/env.h"

#include "rocks/ctypes.hpp"


using namespace rocksdb;

using std::shared_ptr;

extern "C" {
  rocks_writebatch_t* rocks_writebatch_create() {
    return new rocks_writebatch_t;
  }

  rocks_writebatch_t* rocks_writebatch_create_with_reserved_bytes(size_t size) {
    rocks_writebatch_t* b = new rocks_writebatch_t;
    b->rep = WriteBatch(size);
    return b;
  }

  rocks_writebatch_t* rocks_writebatch_create_from(const char* rep,
                                                   size_t size) {
    rocks_writebatch_t* b = new rocks_writebatch_t;
    b->rep = WriteBatch(std::string(rep, size));
    return b;
  }

  void rocks_writebatch_destroy(rocks_writebatch_t* b) {
    delete b;
  }

  void rocks_writebatch_clear(rocks_writebatch_t* b) {
    b->rep.Clear();
  }

  int rocks_writebatch_count(rocks_writebatch_t* b) {
    return b->rep.Count();
  }

  void rocks_writebatch_put(
                            rocks_writebatch_t* b,
                            const char* key, size_t klen,
                            const char* val, size_t vlen) {
    b->rep.Put(Slice(key, klen), Slice(val, vlen));
  }

  void rocks_writebatch_put_cf(
                               rocks_writebatch_t* b,
                               rocks_column_family_handle_t* column_family,
                               const char* key, size_t klen,
                               const char* val, size_t vlen) {
    b->rep.Put(column_family->rep, Slice(key, klen), Slice(val, vlen));
  }

  void rocks_writebatch_putv(
                             rocks_writebatch_t* b,
                             int num_keys, const char* const* keys_list,
                             const size_t* keys_list_sizes,
                             int num_values, const char* const* values_list,
                             const size_t* values_list_sizes) {
    std::vector<Slice> key_slices(num_keys);
    for (int i = 0; i < num_keys; i++) {
      key_slices[i] = Slice(keys_list[i], keys_list_sizes[i]);
    }
    std::vector<Slice> value_slices(num_values);
    for (int i = 0; i < num_values; i++) {
      value_slices[i] = Slice(values_list[i], values_list_sizes[i]);
    }
    b->rep.Put(SliceParts(key_slices.data(), num_keys),
               SliceParts(value_slices.data(), num_values));
  }

  void rocks_writebatch_putv_cf(
                                rocks_writebatch_t* b,
                                rocks_column_family_handle_t* column_family,
                                int num_keys, const char* const* keys_list,
                                const size_t* keys_list_sizes,
                                int num_values, const char* const* values_list,
                                const size_t* values_list_sizes) {
    std::vector<Slice> key_slices(num_keys);
    for (int i = 0; i < num_keys; i++) {
      key_slices[i] = Slice(keys_list[i], keys_list_sizes[i]);
    }
    std::vector<Slice> value_slices(num_values);
    for (int i = 0; i < num_values; i++) {
      value_slices[i] = Slice(values_list[i], values_list_sizes[i]);
    }
    b->rep.Put(column_family->rep, SliceParts(key_slices.data(), num_keys),
               SliceParts(value_slices.data(), num_values));
  }

  void rocks_writebatch_merge(
                              rocks_writebatch_t* b,
                              const char* key, size_t klen,
                              const char* val, size_t vlen) {
    b->rep.Merge(Slice(key, klen), Slice(val, vlen));
  }

  void rocks_writebatch_merge_cf(
                                 rocks_writebatch_t* b,
                                 rocks_column_family_handle_t* column_family,
                                 const char* key, size_t klen,
                                 const char* val, size_t vlen) {
    b->rep.Merge(column_family->rep, Slice(key, klen), Slice(val, vlen));
  }

  void rocks_writebatch_mergev(
                               rocks_writebatch_t* b,
                               int num_keys, const char* const* keys_list,
                               const size_t* keys_list_sizes,
                               int num_values, const char* const* values_list,
                               const size_t* values_list_sizes) {
    std::vector<Slice> key_slices(num_keys);
    for (int i = 0; i < num_keys; i++) {
      key_slices[i] = Slice(keys_list[i], keys_list_sizes[i]);
    }
    std::vector<Slice> value_slices(num_values);
    for (int i = 0; i < num_values; i++) {
      value_slices[i] = Slice(values_list[i], values_list_sizes[i]);
    }
    b->rep.Merge(SliceParts(key_slices.data(), num_keys),
                 SliceParts(value_slices.data(), num_values));
  }

  void rocks_writebatch_mergev_cf(
                                  rocks_writebatch_t* b,
                                  rocks_column_family_handle_t* column_family,
                                  int num_keys, const char* const* keys_list,
                                  const size_t* keys_list_sizes,
                                  int num_values, const char* const* values_list,
                                  const size_t* values_list_sizes) {
    std::vector<Slice> key_slices(num_keys);
    for (int i = 0; i < num_keys; i++) {
      key_slices[i] = Slice(keys_list[i], keys_list_sizes[i]);
    }
    std::vector<Slice> value_slices(num_values);
    for (int i = 0; i < num_values; i++) {
      value_slices[i] = Slice(values_list[i], values_list_sizes[i]);
    }
    b->rep.Merge(column_family->rep, SliceParts(key_slices.data(), num_keys),
                 SliceParts(value_slices.data(), num_values));
  }

  void rocks_writebatch_delete(
                               rocks_writebatch_t* b,
                               const char* key, size_t klen) {
    b->rep.Delete(Slice(key, klen));
  }

  void rocks_writebatch_delete_cf(
                                  rocks_writebatch_t* b,
                                  rocks_column_family_handle_t* column_family,
                                  const char* key, size_t klen) {
    b->rep.Delete(column_family->rep, Slice(key, klen));
  }

  void rocks_writebatch_deletev(
                                rocks_writebatch_t* b,
                                int num_keys, const char* const* keys_list,
                                const size_t* keys_list_sizes) {
    std::vector<Slice> key_slices(num_keys);
    for (int i = 0; i < num_keys; i++) {
      key_slices[i] = Slice(keys_list[i], keys_list_sizes[i]);
    }
    b->rep.Delete(SliceParts(key_slices.data(), num_keys));
  }

  void rocks_writebatch_deletev_cf(
                                   rocks_writebatch_t* b,
                                   rocks_column_family_handle_t* column_family,
                                   int num_keys, const char* const* keys_list,
                                   const size_t* keys_list_sizes) {
    std::vector<Slice> key_slices(num_keys);
    for (int i = 0; i < num_keys; i++) {
      key_slices[i] = Slice(keys_list[i], keys_list_sizes[i]);
    }
    b->rep.Delete(column_family->rep, SliceParts(key_slices.data(), num_keys));
  }

  void rocks_writebatch_single_delete(
                               rocks_writebatch_t* b,
                               const char* key, size_t klen) {
    b->rep.SingleDelete(Slice(key, klen));
  }

  void rocks_writebatch_single_delete_cf(
                                  rocks_writebatch_t* b,
                                  rocks_column_family_handle_t* column_family,
                                  const char* key, size_t klen) {
    b->rep.SingleDelete(column_family->rep, Slice(key, klen));
  }

  void rocks_writebatch_delete_range(rocks_writebatch_t* b,
                                     const char* start_key,
                                     size_t start_key_len, const char* end_key,
                                     size_t end_key_len) {
    b->rep.DeleteRange(Slice(start_key, start_key_len),
                       Slice(end_key, end_key_len));
  }

  void rocks_writebatch_delete_range_cf(
                                        rocks_writebatch_t* b, rocks_column_family_handle_t* column_family,
                                        const char* start_key, size_t start_key_len, const char* end_key,
                                        size_t end_key_len) {
    b->rep.DeleteRange(column_family->rep, Slice(start_key, start_key_len),
                       Slice(end_key, end_key_len));
  }

  void rocks_writebatch_delete_rangev(rocks_writebatch_t* b, int num_keys,
                                      const char* const* start_keys_list,
                                      const size_t* start_keys_list_sizes,
                                      const char* const* end_keys_list,
                                      const size_t* end_keys_list_sizes) {
    std::vector<Slice> start_key_slices(num_keys);
    std::vector<Slice> end_key_slices(num_keys);
    for (int i = 0; i < num_keys; i++) {
      start_key_slices[i] = Slice(start_keys_list[i], start_keys_list_sizes[i]);
      end_key_slices[i] = Slice(end_keys_list[i], end_keys_list_sizes[i]);
    }
    b->rep.DeleteRange(SliceParts(start_key_slices.data(), num_keys),
                       SliceParts(end_key_slices.data(), num_keys));
  }

  void rocks_writebatch_delete_rangev_cf(
                                         rocks_writebatch_t* b, rocks_column_family_handle_t* column_family,
                                         int num_keys, const char* const* start_keys_list,
                                         const size_t* start_keys_list_sizes, const char* const* end_keys_list,
                                         const size_t* end_keys_list_sizes) {
    std::vector<Slice> start_key_slices(num_keys);
    std::vector<Slice> end_key_slices(num_keys);
    for (int i = 0; i < num_keys; i++) {
      start_key_slices[i] = Slice(start_keys_list[i], start_keys_list_sizes[i]);
      end_key_slices[i] = Slice(end_keys_list[i], end_keys_list_sizes[i]);
    }
    b->rep.DeleteRange(column_family->rep,
                       SliceParts(start_key_slices.data(), num_keys),
                       SliceParts(end_key_slices.data(), num_keys));
  }

  void rocks_writebatch_put_log_data(
                                     rocks_writebatch_t* b,
                                     const char* blob, size_t len) {
    b->rep.PutLogData(Slice(blob, len));
  }

  void rocks_writebatch_iterate(
                                rocks_writebatch_t* b,
                                void* state,
                                void (*put)(void*, const char* k, size_t klen, const char* v, size_t vlen),
                                void (*deleted)(void*, const char* k, size_t klen)) {
    class H : public WriteBatch::Handler {
    public:
      void* state_;
      void (*put_)(void*, const char* k, size_t klen, const char* v, size_t vlen);
      void (*deleted_)(void*, const char* k, size_t klen);
      virtual void Put(const Slice& key, const Slice& value) override {
        (*put_)(state_, key.data(), key.size(), value.data(), value.size());
      }
      virtual void Delete(const Slice& key) override {
        (*deleted_)(state_, key.data(), key.size());
      }
    };
    H handler;
    handler.state_ = state;
    handler.put_ = put;
    handler.deleted_ = deleted;
    b->rep.Iterate(&handler);
  }

  const char* rocks_writebatch_data(rocks_writebatch_t* b, size_t* size) {
    *size = b->rep.GetDataSize();
    return b->rep.Data().c_str();
  }

  void rocks_writebatch_set_save_point(rocks_writebatch_t* b) {
    b->rep.SetSavePoint();
  }

  void rocks_writebatch_rollback_to_save_point(rocks_writebatch_t* b,
                                               rocks_status_t* status) {
    SaveError(status, b->rep.RollbackToSavePoint());
  }

  rocks_writebatch_t* rocks_writebatch_copy(rocks_writebatch_t* b) {
    return new rocks_writebatch_t { WriteBatch(b->rep) };
  }
}
