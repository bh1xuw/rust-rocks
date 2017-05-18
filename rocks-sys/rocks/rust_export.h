
#include "rocksdb/slice.h"
#include "rocksdb/env.h"

#include <cstdint>

using rocksdb::Slice;
using rocksdb::Logger;

#ifdef __cplusplus
extern "C" {
#endif


  extern void rust_hello_world();

  extern void rust_drop_vec_u8(char* op, size_t len);

  extern int32_t rust_associative_merge_operator_call(
                                                    void* op,
                                                    const Slice* key,
                                                    const Slice* existing_value,
                                                    const Slice* value,
                                                    char** new_value, size_t* new_value_len,
                                                    Logger* logger);

  extern const char* rust_associative_merge_operator_name(void* op);

#ifdef __cplusplus
}
#endif
