
#include "rocksdb/status.h"

#include "rocks/ctypes.hpp"

using namespace rocksdb;

extern "C" {

rocks_status_t* rocks_status_create() { return new rocks_status_t; }

void rocks_status_destroy(rocks_status_t* s) { delete s; }

// TODO: handle code
rocks_status_t* rocks_status_create_with_code_and_msg(int code, const char* msg, size_t len) {
  auto ccode = static_cast<Status::Code>(code);
  auto message = Slice(msg, len);
  return new rocks_status_t{Status::InvalidArgument(message)};
}

int rocks_status_code(rocks_status_t* s) { return s->rep.code(); }

int rocks_status_subcode(rocks_status_t* s) { return s->rep.subcode(); }

const char* rocks_status_get_state(rocks_status_t* s) { return s->rep.getState(); }
}
