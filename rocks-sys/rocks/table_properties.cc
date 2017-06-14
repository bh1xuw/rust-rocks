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
