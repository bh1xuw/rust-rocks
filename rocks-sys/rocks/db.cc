#include "rocksdb/db.h"
#include "rocks/ctypes.hpp"

using namespace rocksdb;

using std::shared_ptr;

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
                                                                 const rocks_options_t* column_family_options,
                                                                 const char* column_family_name,
                                                                 rocks_status_t* status) {
    rocks_column_family_handle_t* handle = new rocks_column_family_handle_t;
    SaveError(status,
              db->rep->CreateColumnFamily(ColumnFamilyOptions(column_family_options->rep),
                                          std::string(column_family_name), &(handle->rep)));
    return handle;
  }

  void rocks_db_drop_column_family(
                                   rocks_db_t* db,
                                   rocks_column_family_handle_t* handle,
                                   rocks_status_t* status) {
    SaveError(status, db->rep->DropColumnFamily(handle->rep));
  }

  void rocks_db_column_family_handle_destroy(rocks_column_family_handle_t* handle) {
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

  // merge...

  // write...





  char* rocks_db_get(
                    rocks_db_t* db,
                    const rocks_readoptions_t* options,
                    const char* key, size_t keylen,
                    size_t* vallen,
                    rocks_status_t* status) {
    char* result = nullptr;
    std::string tmp;
    Status s = db->rep->Get(options->rep, Slice(key, keylen), &tmp);
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

