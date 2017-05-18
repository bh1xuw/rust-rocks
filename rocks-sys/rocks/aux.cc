#include <string>
#include <vector>

#include "rocksdb/slice.h"

using namespace rocksdb;

using std::shared_ptr;

extern "C" {
  size_t cxx_vector_slice_size(const void* list) {
    auto p = reinterpret_cast<const std::vector<Slice>*>(list);
    return p->size();
  }


  const void* cxx_vector_slice_nth(const void* list, size_t n) {
    auto p = reinterpret_cast<const std::vector<Slice>*>(list);
    return (void *)&p->at(n);
  }

  void cxx_string_assign(void* s, const char* p, size_t len) {
    auto str = reinterpret_cast<std::string*>(s);
    str->assign(p, len);
  }

  const char* cxx_string_data(const void *s) {
    auto str = reinterpret_cast<const std::string*>(s);
    return str->data();
  }

  size_t cxx_string_size(const void *s) {
    auto str = reinterpret_cast<const std::string*>(s);
    return str->size();
  }
}
