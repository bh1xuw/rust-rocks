
#include "rocksdb/status.h"

#include "rocks/ctypes.hpp"

using namespace rocksdb;

extern "C" {

void rocks_status_destroy(rocks_status_t* s) { delete s; }

int rocks_status_code(rocks_status_t* s) { return s->rep.code(); }

int rocks_status_subcode(rocks_status_t* s) { return s->rep.subcode(); }

int rocks_status_severity(rocks_status_t* s) { return s->rep.severity(); }

const char* rocks_status_get_state(rocks_status_t* s) { return s->rep.getState(); }
}
