
#include "rocksdb/status.h"

#include "rocks/ctypes.hpp"

using namespace rocksdb;

extern "C" {

  void rocks_status_convert(const Status *status, rocks_status_t *p) {
    if (p != nullptr) {
      p->code = status->code();
      p->sub_code = status->subcode();
      p->state = status->getState();
    }
  }
}


