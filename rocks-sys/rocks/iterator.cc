#include "rocksdb/env.h"

#include "rocks/ctypes.hpp"


using namespace rocksdb;

using std::shared_ptr;

extern "C" {
  void rocks_iter_destroy(rocks_iterator_t* iter) {
    delete iter->rep;
    delete iter;
  }

  unsigned char rocks_iter_valid(const rocks_iterator_t* iter) {
    return iter->rep->Valid();
  }

  void rocks_iter_seek_to_first(rocks_iterator_t* iter) {
    iter->rep->SeekToFirst();
  }

  void rocks_iter_seek_to_last(rocks_iterator_t* iter) {
    iter->rep->SeekToLast();
  }

  void rocks_iter_seek(rocks_iterator_t* iter, const char* k, size_t klen) {
    iter->rep->Seek(Slice(k, klen));
  }

  void rocks_iter_seek_for_prev(rocks_iterator_t* iter, const char* k,
                                  size_t klen) {
    iter->rep->SeekForPrev(Slice(k, klen));
  }

  void rocks_iter_next(rocks_iterator_t* iter) {
    iter->rep->Next();
  }

  void rocks_iter_prev(rocks_iterator_t* iter) {
    iter->rep->Prev();
  }

  const char* rocks_iter_key(const rocks_iterator_t* iter, size_t* klen) {
    Slice s = iter->rep->key();
    *klen = s.size();
    return s.data();
  }

  const char* rocks_iter_value(const rocks_iterator_t* iter, size_t* vlen) {
    Slice s = iter->rep->value();
    *vlen = s.size();
    return s.data();
  }

  void rocks_iter_get_status(const rocks_iterator_t* iter, rocks_status_t* status) {
    SaveError(status, iter->rep->status());
  }
}
