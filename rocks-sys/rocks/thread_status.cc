#include "rocksdb/thread_status.h"

#include "rocks/ctypes.hpp"

using namespace rocksdb;

extern "C" {
void rocks_thread_status_destroy(rocks_thread_status_t* status) { delete status; }

uint64_t rocks_thread_status_get_thread_id(const rocks_thread_status_t* status) { return status->rep.thread_id; }

int rocks_thread_status_get_thread_type(const rocks_thread_status_t* status) {
  return static_cast<int>(status->rep.thread_type);
}

const char* rocks_thread_status_get_db_name(const rocks_thread_status_t* status, size_t* len) {
  *len = status->rep.db_name.size();
  return status->rep.db_name.data();
}

const char* rocks_thread_status_get_cf_name(const rocks_thread_status_t* status, size_t* len) {
  *len = status->rep.cf_name.size();
  return status->rep.cf_name.data();
}

int rocks_thread_status_get_operation_type(const rocks_thread_status_t* status) {
  return static_cast<int>(status->rep.operation_type);
}

uint64_t rocks_thread_status_get_op_elapsed_micros(const rocks_thread_status_t* status) {
  return status->rep.op_elapsed_micros;
}

int rocks_thread_status_get_operation_stage(const rocks_thread_status_t* status) {
  return static_cast<int>(status->rep.operation_stage);
}

const uint64_t* rocks_thread_status_get_op_properties(const rocks_thread_status_t* status, size_t* len) {
  *len = ThreadStatus::kNumOperationProperties;
  return &status->rep.op_properties[0];
}

int rocks_thread_status_get_state_type(const rocks_thread_status_t* status) {
  return static_cast<int>(status->rep.state_type);
}
}
