#include <unordered_map>

#include "rocks/ctypes.hpp"

using namespace rocksdb;

// TablePropertiesCollection, used in db.h
// since this is a readonly structure

extern "C" {

void rocks_table_props_collection_destroy(
    rocks_table_props_collection_t* coll) {
  delete coll;
}

void rocks_table_props_destroy(rocks_table_props_t* props) { delete props; }

void rocks_table_props_collection_iter_destroy(
    rocks_table_props_collection_iter_t* it) {
  delete it;
}

void rocks_user_collected_props_iter_destroy(
    rocks_user_collected_props_iter_t* it) {
  delete it;
}

size_t rocks_table_props_collection_size(rocks_table_props_collection_t* coll) {
  return coll->rep.size();
}

rocks_table_props_t* rocks_table_props_collection_at(
    rocks_table_props_collection_t* coll, const char* key_ptr, size_t key_len) {
  auto key = std::string(key_ptr, key_len);
  auto search = coll->rep.find(key);
  if (search != coll->rep.end()) {
    auto props = new rocks_table_props_t{search->second};
    return props;
  }
  return nullptr;
}

rocks_table_props_collection_iter_t* rocks_table_props_collection_iter_create(
    rocks_table_props_collection_t* coll) {
  if (!coll->rep.empty()) {
    return new rocks_table_props_collection_iter_t{coll->rep.cbegin(),
                                                   coll->rep.cend()};
  }
  return nullptr;
}

unsigned char rocks_table_props_collection_iter_next(
    rocks_table_props_collection_iter_t* it) {
  it->rep++;
  return it->rep != it->cend;
}

void rocks_table_props_collection_iter_key(
    rocks_table_props_collection_iter_t* it, void* s) {
  auto key = it->rep->first;
  rust_string_assign(s, key.data(), key.size());
}

rocks_table_props_t* rocks_table_props_collection_iter_value(
    rocks_table_props_collection_iter_t* it) {
  return new rocks_table_props_t{it->rep->second};
}

uint64_t rocks_table_props_get_data_size(rocks_table_props_t* prop) {
  return prop->rep->data_size;
}
uint64_t rocks_table_props_get_index_size(rocks_table_props_t* prop) {
  return prop->rep->index_size;
}
uint64_t rocks_table_props_get_filter_size(rocks_table_props_t* prop) {
  return prop->rep->filter_size;
}
uint64_t rocks_table_props_get_raw_key_size(rocks_table_props_t* prop) {
  return prop->rep->raw_key_size;
}
uint64_t rocks_table_props_get_raw_value_size(rocks_table_props_t* prop) {
  return prop->rep->raw_value_size;
}
uint64_t rocks_table_props_get_num_data_blocks(rocks_table_props_t* prop) {
  return prop->rep->num_data_blocks;
}
uint64_t rocks_table_props_get_num_entries(rocks_table_props_t* prop) {
  return prop->rep->num_entries;
}
uint64_t rocks_table_props_get_format_version(rocks_table_props_t* prop) {
  return prop->rep->format_version;
}
uint64_t rocks_table_props_get_fixed_key_len(rocks_table_props_t* prop) {
  return prop->rep->fixed_key_len;
}
uint32_t rocks_table_props_get_column_family_id(rocks_table_props_t* prop) {
  return prop->rep->column_family_id;
}
const char* rocks_table_props_get_column_family_name(rocks_table_props_t* prop,
                                                     size_t* len) {
  *len = prop->rep->column_family_name.size();
  return prop->rep->column_family_name.data();
}
const char* rocks_table_props_get_filter_policy_name(rocks_table_props_t* prop,
                                                     size_t* len) {
  *len = prop->rep->filter_policy_name.size();
  return prop->rep->filter_policy_name.data();
}
const char* rocks_table_props_get_comparator_name(rocks_table_props_t* prop,
                                                  size_t* len) {
  *len = prop->rep->comparator_name.size();
  return prop->rep->comparator_name.data();
}
const char* rocks_table_props_get_merge_operator_name(rocks_table_props_t* prop,
                                                      size_t* len) {
  *len = prop->rep->merge_operator_name.size();
  return prop->rep->merge_operator_name.data();
}
const char* rocks_table_props_get_prefix_extractor_name(
    rocks_table_props_t* prop, size_t* len) {
  *len = prop->rep->prefix_extractor_name.size();
  return prop->rep->prefix_extractor_name.data();
}
const char* rocks_table_props_get_property_collectors_names(
    rocks_table_props_t* prop, size_t* len) {
  *len = prop->rep->property_collectors_names.size();
  return prop->rep->property_collectors_names.data();
}
const char* rocks_table_props_get_compression_name(rocks_table_props_t* prop,
                                                   size_t* len) {
  *len = prop->rep->compression_name.size();
  return prop->rep->compression_name.data();
}

void rocks_table_props_to_string(rocks_table_props_t* prop, void* s) {
  auto str = prop->rep->ToString();
  rust_string_assign(s, str.data(), str.size());
}

rocks_user_collected_props_t* rocks_table_props_get_user_collected_properties(
    rocks_table_props_t* prop) {
  auto ptr =
      reinterpret_cast<const void*>(&prop->rep->user_collected_properties);
  // const pointer to non-const
  return (rocks_user_collected_props_t*)ptr;
}

rocks_user_collected_props_t* rocks_table_props_get_readable_properties(
    rocks_table_props_t* prop) {
  auto ptr = reinterpret_cast<const void*>(&prop->rep->readable_properties);
  // const pointer to non-const
  return (rocks_user_collected_props_t*)ptr;
}

void rocks_user_collected_props_insert(rocks_user_collected_props_t* prop,
                                       const char* key_ptr, size_t key_len,
                                       const char* val_ptr, size_t val_len) {
  auto user_prop = reinterpret_cast<UserCollectedProperties*>(prop);
  (*user_prop)[std::string(key_ptr, key_len)] = std::string(val_ptr, val_len);
}

size_t rocks_user_collected_props_size(rocks_user_collected_props_t* prop) {
  auto user_prop = reinterpret_cast<UserCollectedProperties*>(prop);
  return user_prop->size();
}

rocks_user_collected_props_iter_t* rocks_user_collected_props_iter_create(
    rocks_user_collected_props_t* prop) {
  auto user_prop = reinterpret_cast<UserCollectedProperties*>(prop);
  if (!user_prop->empty()) {
    return new rocks_user_collected_props_iter_t{user_prop->cbegin(),
                                                 user_prop->cend()};
  }
  return nullptr;
}

unsigned char rocks_user_collected_props_iter_next(
    rocks_user_collected_props_iter_t* it) {
  it->rep++;
  return it->rep != it->cend;
}

void rocks_user_collected_props_iter_key(rocks_user_collected_props_iter_t* it,
                                         void* s) {  // String
  auto key = it->rep->first;
  rust_string_assign(s, key.data(), key.size());
}

void rocks_user_collected_props_iter_value(
    rocks_user_collected_props_iter_t* it,
    void* v) {  // Vec<u8>
  auto value = it->rep->second;
  rust_vec_u8_assign(v, value.data(), value.size());
}
}
