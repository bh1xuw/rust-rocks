#include "rocksdb/db.h"
#include "rocks/ctypes.hpp"

#include <iostream>

using namespace rocksdb;

using std::shared_ptr;

extern "C" {
  const char* rocks_column_family_handle_get_name(const rocks_column_family_handle_t* handle) {
    return handle->rep->GetName().c_str();
  }

  uint32_t rocks_column_family_handle_get_id(const rocks_column_family_handle_t* handle) {
    return handle->rep->GetID();
  }
}


extern "C" {

  // DB
  rocks_db_t* rocks_db_open(
                            const rocks_options_t* options,
                            const char* name,
                            rocks_status_t* status) {
    DB* db = nullptr;
    Status st = DB::Open(options->rep, std::string(name), &db);
    rocks_status_convert(&st, status);
    if (st.ok()) {
      rocks_db_t* result = new rocks_db_t;
      result->rep = db;
      return result;
    }
    return nullptr;
  }

  void rocks_db_close(rocks_db_t* db) {
    delete db->rep;
    delete db;
  }

  rocks_db_t* rocks_db_open_for_read_only(
                                          const rocks_options_t* options,
                                          const char* name,
                                          unsigned char error_if_log_file_exist,
                                          rocks_status_t* status) {
    DB* db;
    auto st = DB::OpenForReadOnly(options->rep, std::string(name), &db, error_if_log_file_exist);
    rocks_status_convert(&st, status);

    if (st.ok()) {
      rocks_db_t* result = new rocks_db_t;
      result->rep = db;
      return result;
    }
    return nullptr;
  }

  rocks_db_t* rocks_db_open_column_families(
                                            const rocks_options_t* db_options,
                                            const char* name,
                                            int num_column_families,
                                            const char* const* column_family_names,
                                            const rocks_cfoptions_t* const* column_family_options,
                                            rocks_column_family_handle_t** column_family_handles,
                                            rocks_status_t *status) {
    std::vector<ColumnFamilyDescriptor> column_families;
    for (int i = 0; i < num_column_families; i++) {
      column_families.push_back(ColumnFamilyDescriptor(
                                                       std::string(column_family_names[i]),
                                                       ColumnFamilyOptions(column_family_options[i]->rep)));
    }

    DB* db;
    std::vector<ColumnFamilyHandle*> handles;
    if (SaveError(status, DB::Open(DBOptions(db_options->rep),
                                   std::string(name), column_families, &handles, &db))) {
      return nullptr;
    }

    for (size_t i = 0; i < handles.size(); i++) {
      rocks_column_family_handle_t* c_handle = new rocks_column_family_handle_t;
      c_handle->rep = handles[i];
      column_family_handles[i] = c_handle;
    }
    rocks_db_t* result = new rocks_db_t;
    result->rep = db;
    return result;
  }

  rocks_db_t* rocks_db_open_for_read_only_column_families(
                                                          const rocks_options_t* db_options,
                                                          const char* name,
                                                          int num_column_families,
                                                          const char* const* column_family_names,
                                                          const rocks_cfoptions_t* const* column_family_options,
                                                          rocks_column_family_handle_t** column_family_handles,
                                                          unsigned char error_if_log_file_exist,
                                                          rocks_status_t *status) {
    std::vector<ColumnFamilyDescriptor> column_families;
    for (int i = 0; i < num_column_families; i++) {
      column_families.push_back(ColumnFamilyDescriptor(
                                                       std::string(column_family_names[i]),
                                                       ColumnFamilyOptions(column_family_options[i]->rep)));
    }

    DB* db;
    std::vector<ColumnFamilyHandle*> handles;
    if (SaveError(status, DB::OpenForReadOnly(DBOptions(db_options->rep),
                                              std::string(name), column_families, &handles, &db, error_if_log_file_exist))) {
      return nullptr;
    }

    for (size_t i = 0; i < handles.size(); i++) {
      rocks_column_family_handle_t* c_handle = new rocks_column_family_handle_t;
      c_handle->rep = handles[i];
      column_family_handles[i] = c_handle;
    }
    rocks_db_t* result = new rocks_db_t;
    result->rep = db;
    return result;
  }

  char** rocks_db_list_column_families(
                                       const rocks_options_t* options,
                                       const char* name,
                                       size_t* lencfs,
                                       rocks_status_t* status) {
    std::vector<std::string> fams;
    auto st = DB::ListColumnFamilies(DBOptions(options->rep),
                                     std::string(name), &fams);
    rocks_status_convert(&st, status);
    if (!st.ok()) {
      *lencfs = 0;
      return nullptr;
    }

    *lencfs = fams.size();
    char** column_families = static_cast<char**>(malloc(sizeof(char*) * fams.size()));
    for (size_t i = 0; i < fams.size(); i++) {
      column_families[i] = strdup(fams[i].c_str());
    }
    return column_families;
  }

  void rocks_db_list_column_families_destroy(char** list, size_t len) {
    if (list == nullptr) return ;
    for (size_t i = 0; i < len; ++i) {
      free(list[i]);
    }
    free(list);
  }

  rocks_column_family_handle_t* rocks_db_create_column_family(
                                                              rocks_db_t* db,
                                                              const rocks_cfoptions_t* column_family_options,
                                                              const char* column_family_name,
                                                              rocks_status_t* status) {
    rocks_column_family_handle_t* handle = new rocks_column_family_handle_t;
    SaveError(status,
              db->rep->CreateColumnFamily(ColumnFamilyOptions(column_family_options->rep),
                                          std::string(column_family_name), &(handle->rep)));
    return handle;
  }

  void rocks_db_drop_column_family(rocks_db_t* db,
                                   rocks_column_family_handle_t* handle,
                                   rocks_status_t* status) {
    SaveError(status, db->rep->DropColumnFamily(handle->rep));
  }

  void rocks_db_destroy_column_family_handle(rocks_db_t* db,
                                             rocks_column_family_handle_t* handle,
                                             rocks_status_t* status) {
    SaveError(status, db->rep->DestroyColumnFamilyHandle(handle->rep));
  }


  void rocks_column_family_handle_destroy(rocks_column_family_handle_t* handle) {
    delete handle->rep;
    delete handle;
  }

  void rocks_db_put(
                    rocks_db_t* db,
                    const rocks_writeoptions_t* options,
                    const char* key, size_t keylen,
                    const char* val, size_t vallen,
                    rocks_status_t* status) {
    SaveError(status,
              db->rep->Put(options->rep, Slice(key, keylen), Slice(val, vallen)));
  }

  /*
  void rocks_db_put_slice(
                    rocks_db_t* db,
                    const rocks_writeoptions_t* options,
                    const Slice* key, const Slice* value,
                    rocks_status_t* status) {
    SaveError(status,
              db->rep->Put(options->rep, *key, *value));
              }*/


  void rocks_db_put_cf(
                       rocks_db_t* db,
                       const rocks_writeoptions_t* options,
                       rocks_column_family_handle_t* column_family,
                       const char* key, size_t keylen,
                       const char* val, size_t vallen,
                       rocks_status_t* status) {
    SaveError(status,
              db->rep->Put(options->rep, column_family->rep,
                           Slice(key, keylen), Slice(val, vallen)));
  }

  void rocks_db_delete(
                       rocks_db_t* db,
                       const rocks_writeoptions_t* options,
                       const char* key, size_t keylen,
                       rocks_status_t* status) {
    SaveError(status, db->rep->Delete(options->rep, Slice(key, keylen)));
  }

  void rocks_db_delete_cf(
                          rocks_db_t* db,
                          const rocks_writeoptions_t* options,
                          rocks_column_family_handle_t* column_family,
                          const char* key, size_t keylen,
                          rocks_status_t* status) {
    SaveError(status, db->rep->Delete(options->rep, column_family->rep,
                                      Slice(key, keylen)));
  }


  void rocks_db_single_delete(
                              rocks_db_t* db,
                              const rocks_writeoptions_t* options,
                              const char* key, size_t keylen,
                              rocks_status_t* status) {
    SaveError(status, db->rep->SingleDelete(options->rep, Slice(key, keylen)));
  }

  void rocks_db_single_delete_cf(
                                 rocks_db_t* db,
                                 const rocks_writeoptions_t* options,
                                 rocks_column_family_handle_t* column_family,
                                 const char* key, size_t keylen,
                                 rocks_status_t* status) {
    SaveError(status, db->rep->SingleDelete(options->rep, column_family->rep,
                                            Slice(key, keylen)));
  }

  void rocks_db_delete_range_cf(
                                rocks_db_t* db,
                                const rocks_writeoptions_t* options,
                                rocks_column_family_handle_t* column_family,
                                const char* begin_key, size_t begin_keylen,
                                const char* end_key, size_t end_keylen,
                                rocks_status_t* status) {
    SaveError(status, db->rep->DeleteRange(options->rep, column_family->rep,
                                           Slice(begin_key, begin_keylen), Slice(end_key, end_keylen)));
  }

  void rocks_db_merge(
                      rocks_db_t* db,
                      const rocks_writeoptions_t* options,
                      const char* key, size_t keylen,
                      const char* val, size_t vallen,
                      rocks_status_t* status) {
    SaveError(status,
              db->rep->Merge(options->rep, Slice(key, keylen), Slice(val, vallen)));
  }

  void rocks_db_merge_cf(
                         rocks_db_t* db,
                         const rocks_writeoptions_t* options,
                         rocks_column_family_handle_t* column_family,
                         const char* key, size_t keylen,
                         const char* val, size_t vallen,
                         rocks_status_t* status) {
    SaveError(status,
              db->rep->Merge(options->rep, column_family->rep,
                             Slice(key, keylen), Slice(val, vallen)));
  }

  void rocks_db_write(
                      rocks_db_t* db,
                      const rocks_writeoptions_t* options,
                      rocks_writebatch_t* batch,
                      rocks_status_t* status) {
    SaveError(status, db->rep->Write(options->rep, &batch->rep));
  }

  char* rocks_db_get(
                     rocks_db_t* db,
                     const rocks_readoptions_t* options,
                     const char* key, size_t keylen,
                     size_t* vallen,
                     rocks_status_t* status) {
    char* result = nullptr;
    std::string tmp;
    Status s = db->rep->Get(options->rep, Slice(key, keylen), &tmp);
    SaveError(status, s);

    if (s.ok()) {
      *vallen = tmp.size();
      result = CopyString(tmp);
    } else {
      *vallen = 0;
      if (!s.IsNotFound()) {
        SaveError(status, s);
      }
    }
    return result;
  }

  char* rocks_db_get_cf(
                        rocks_db_t* db,
                        const rocks_readoptions_t* options,
                        rocks_column_family_handle_t* column_family,
                        const char* key, size_t keylen,
                        size_t* vallen,
                        rocks_status_t* status) {
    char* result = nullptr;
    std::string tmp;
    Status s = db->rep->Get(options->rep, column_family->rep,
                            Slice(key, keylen), &tmp);
    if (s.ok()) {
      *vallen = tmp.size();
      result = CopyString(tmp);
    } else {
      *vallen = 0;
      if (!s.IsNotFound()) {
        SaveError(status, s);
      }
    }
    return result;
  }

  void rocks_db_multi_get(
                          rocks_db_t* db,
                          const rocks_readoptions_t* options,
                          size_t num_keys, const char* const* keys_list,
                          const size_t* keys_list_sizes,
                          char** values_list, size_t* values_list_sizes,
                          rocks_status_t* status) {
    std::vector<Slice> keys(num_keys);
    for (size_t i = 0; i < num_keys; i++) {
      keys[i] = Slice(keys_list[i], keys_list_sizes[i]);
    }
    std::vector<std::string> values(num_keys);
    std::vector<Status> statuses = db->rep->MultiGet(options->rep, keys, &values);
    for (size_t i = 0; i < num_keys; i++) {
      rocks_status_convert(&statuses[i], &status[i]);
      if (statuses[i].ok()) {
        values_list[i] = CopyString(values[i]);
        values_list_sizes[i] = values[i].size();
      } else {
        values_list[i] = nullptr;
        values_list_sizes[i] = 0;
      }
    }
  }

  void rocks_db_multi_get_cf(
                             rocks_db_t* db,
                             const rocks_readoptions_t* options,
                             const rocks_column_family_handle_t* const* column_families,
                             size_t num_keys, const char* const* keys_list,
                             const size_t* keys_list_sizes,
                             char** values_list, size_t* values_list_sizes,
                             rocks_status_t* status) {
    std::vector<Slice> keys(num_keys);
    std::vector<ColumnFamilyHandle*> cfs(num_keys);
    for (size_t i = 0; i < num_keys; i++) {
      keys[i] = Slice(keys_list[i], keys_list_sizes[i]);
      cfs[i] = column_families[i]->rep;
    }
    std::vector<std::string> values(num_keys);
    std::vector<Status> statuses = db->rep->MultiGet(options->rep, cfs, keys, &values);
    for (size_t i = 0; i < num_keys; i++) {
      rocks_status_convert(&statuses[i], &status[i]);
      if (statuses[i].ok()) {
        values_list[i] = CopyString(values[i]);
        values_list_sizes[i] = values[i].size();
      } else {
        values_list[i] = nullptr;
        values_list_sizes[i] = 0;
      }
    }
  }

  rocks_iterator_t* rocks_db_create_iterator(
                                              rocks_db_t* db,
                                              const rocks_readoptions_t* options) {
    rocks_iterator_t* result = new rocks_iterator_t;
    result->rep = db->rep->NewIterator(options->rep);
    return result;
  }

  rocks_snapshot_t* rocks_db_get_snapshot(rocks_db_t* db) {
    rocks_snapshot_t* result = new rocks_snapshot_t;
    result->rep = db->rep->GetSnapshot();
    return result;
  }

  // also destroy
  void rocks_db_release_snapshot(rocks_db_t* db, rocks_snapshot_t* snapshot) {
    db->rep->ReleaseSnapshot(snapshot->rep);
    delete snapshot;
  }


  // public functions
  void rocks_destroy_db(
                        const rocks_options_t* options,
                        const char* name,
                        rocks_status_t* status) {
    auto st = DestroyDB(name, options->rep);
    rocks_status_convert(&st, status);
  }

  void rocks_repair_db(
                       const rocks_options_t* options,
                       const char* name,
                       rocks_status_t* status) {
    auto st = RepairDB(name, options->rep);
    rocks_status_convert(&st, status);
  }

}

