#include "rocksdb/snapshot.h"

#include "rocks/ctypes.hpp"

using namespace rocksdb;

using std::shared_ptr;

extern "C" {

const rocks_snapshot_t* rocks_create_snapshot(rocks_db_t* db) {
  rocks_snapshot_t* result = new rocks_snapshot_t;
  result->rep = db->rep->GetSnapshot();
  return result;
}

void rocks_snapshot_destroy(rocks_snapshot_t* snapshot) { delete snapshot; }

void rocks_release_snapshot(rocks_db_t* db, rocks_snapshot_t* snapshot) {
  db->rep->ReleaseSnapshot(snapshot->rep);
  snapshot->rep = nullptr;
  delete snapshot;
}

uint64_t rocks_snapshot_get_sequence_number(rocks_snapshot_t* snapshot) { return snapshot->rep->GetSequenceNumber(); }
}
