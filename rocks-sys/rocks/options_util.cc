#include "rocksdb/utilities/options_util.h"

#include "rocks/ctypes.hpp"

using namespace rocksdb;

#ifdef __cplusplus
extern "C" {
#endif

rocks_column_family_descriptor_t** rocks_load_latest_options(const char* c_dbpath, rocks_dboptions_t* db_options,
                                                             size_t* cf_descs_len, rocks_status_t** status) {
  const std::string dbpath = std::string(c_dbpath);
  std::vector<ColumnFamilyDescriptor> cf_descs;

  auto st = LoadLatestOptions(dbpath, Env::Default(), &db_options->rep, &cf_descs);
  if (SaveError(status, std::move(st))) {
    return nullptr;
  }

  *cf_descs_len = cf_descs.size();
  rocks_column_family_descriptor_t** c_cf_descs = static_cast<rocks_column_family_descriptor_t**>(
      malloc(sizeof(rocks_column_family_descriptor_t*) * cf_descs.size()));
  for (auto i = 0; i < *cf_descs_len; i++) {
    // Use copy constructor. The original ColumnFamilyDescriptor will be freed with the std::vector.
    c_cf_descs[i] = new rocks_column_family_descriptor_t{ColumnFamilyDescriptor(cf_descs[i])};
  }
  return c_cf_descs;
}

void rocks_load_options_destroy_cf_descs(rocks_column_family_descriptor_t** c_cf_descs, size_t len) {
  for (auto i = 0; i < len; i++) {
    delete c_cf_descs[i];
  }
  free(c_cf_descs);
}

#ifdef __cplusplus
}
#endif
