
#include "rocksdb/compaction_filter.h"
#include "rocksdb/env.h"
#include "rocksdb/slice.h"
#include "rocksdb/table_properties.h"

#include <cstdint>
#include <string>

using rocksdb::Slice;
using rocksdb::Logger;
using rocksdb::CompactionFilter;
using rocksdb::UserCollectedProperties;
using rocksdb::TablePropertiesCollectorFactory;

#ifdef __cplusplus
extern "C" {
#endif

extern void rust_hello_world();

extern void rust_drop_vec_u8(char* op, size_t len);

extern void rust_string_assign(void* s, const char* p, size_t len);

extern void rust_vec_u8_assign(void* v, const char* p, size_t len);

/* compaction filter */
extern int rust_compaction_filter_call(void* f, int level,
                                       const Slice* key,  // &&[u8]
                                       CompactionFilter::ValueType value_type,
                                       const Slice* existing_value,  // &&[u8]
                                       std::string* new_value,
                                       std::string* skip_until);

extern const char* rust_compaction_filter_name(void* f);

extern char rust_compaction_filter_ignore_snapshots(void* f);

extern void rust_compaction_filter_drop(void* f);

/* slice transform */
extern void rust_slice_transform_call(void* t, const Slice* key,
                                      char* const* ret, size_t* ret_len);

extern const char* rust_slice_transform_name(void* t);

extern char rust_slice_transform_in_domain(void* t, const Slice* key);

extern void rust_slice_transform_drop(void* t);

/* merge operator*/

extern int32_t rust_associative_merge_operator_call(
    void* op, const Slice* key, const Slice* existing_value, const Slice* value,
    char** new_value, size_t* new_value_len, Logger* logger);

extern const char* rust_associative_merge_operator_name(void* op);

extern void rust_associative_merge_operator_drop(void* op);

extern const char* rust_merge_operator_name(void* op);

extern int32_t rust_merge_operator_call_full_merge_v2(void* op,
                                                      const void* merge_in,
                                                      void* merge_out);

extern void rust_merge_operator_drop(void* op);

/* comparator */

extern int rust_comparator_compare(void* cp, const Slice* a, const Slice* b);

extern char rust_comparator_equal(void* cp, const Slice* a, const Slice* b);

extern const char* rust_comparator_name(const void* cp);

extern void rust_comparator_find_shortest_separator(
    void* cp, std::string* start, /* std::string */
    const Slice* limit);

extern void rust_comparator_find_short_successor(void* cp, std::string* key);

extern void rust_comparator_drop(void* cp);

/* table_properties */

extern void rust_table_props_collector_add_user_key(void* c, const Slice* key,
                                                    const Slice* value,
                                                    int type, uint64_t seq,
                                                    uint64_t file_size);

extern void rust_table_props_collector_finish(void* c,
                                              UserCollectedProperties* props);

extern const char* rust_table_props_collector_name(void* c);

extern unsigned char rust_table_props_collector_need_compact(void* c);

extern void rust_table_props_collector_drop(void* c);

// *mut TablePropertiesCollector
extern void* rust_table_props_collector_factory_new_collector(void* f,
                                                              uint32_t context);

extern const char* rust_table_props_collector_factory_name(void* f);

extern void rust_table_props_collector_factory_drop(void* f);

#ifdef __cplusplus
}
#endif
