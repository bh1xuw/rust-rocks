#include "rocks/ctypes.hpp"

using namespace rocksdb;

using std::shared_ptr;

extern "C" {

rocks_key_version_collection_t* rocks_db_get_all_key_versions(rocks_db_t* db, const char* begin_key,
                                                              size_t begin_keylen, const char* end_key,
                                                              size_t end_keylen, rocks_status_t** status) {
  auto coll = new rocks_key_version_collection_t;
  // FIXME: handle max_num_ikeys
  auto st = GetAllKeyVersions(db->rep, Slice(begin_key, begin_keylen), Slice(end_key, end_keylen), 65535, &coll->rep);

  if (!SaveError(status, std::move(st))) {
    return coll;
  }
  delete coll;
  return nullptr;
}

void rocks_key_version_collection_destroy(rocks_key_version_collection_t* coll) { delete coll; }

size_t rocks_key_version_collection_size(rocks_key_version_collection_t* coll) { return coll->rep.size(); }

rocks_key_version_t* rocks_key_version_collection_nth(rocks_key_version_collection_t* coll, size_t index) {
  return reinterpret_cast<rocks_key_version_t*>(&coll->rep[index]);
}

const char* rocks_key_version_user_key(const rocks_key_version_t* ver, size_t* len) {
  auto key_ver = reinterpret_cast<const KeyVersion*>(ver);
  *len = key_ver->user_key.size();
  return key_ver->user_key.data();
}

const char* rocks_key_version_value(const rocks_key_version_t* ver, size_t* len) {
  auto key_ver = reinterpret_cast<const KeyVersion*>(ver);
  *len = key_ver->value.size();
  return key_ver->value.data();
}

uint64_t rocks_key_version_sequence_numer(const rocks_key_version_t* ver) {
  auto key_ver = reinterpret_cast<const KeyVersion*>(ver);
  return key_ver->sequence;
}

int rocks_key_version_type(const rocks_key_version_t* ver) {
  auto key_ver = reinterpret_cast<const KeyVersion*>(ver);
  return key_ver->type;
}
}
