#include "rocks/ctypes.hpp"

using namespace rocksdb;

using std::shared_ptr;

extern "C" {
rocks_write_buffer_manager_t* rocks_write_buffer_manager_create(
    size_t buffer_size) {
  auto manager = new rocks_write_buffer_manager_t;
  manager->rep.reset(new WriteBufferManager(buffer_size));
  return manager;
}

void rocks_write_buffer_manager_destroy(rocks_write_buffer_manager_t* manager) {
  delete manager;
}

unsigned char rocks_write_buffer_manager_enabled(
    rocks_write_buffer_manager_t* manager) {
  return manager->rep->enabled();
}

size_t rocks_write_buffer_manager_memory_usage(
    rocks_write_buffer_manager_t* manager) {
  return manager->rep->memory_usage();
}

size_t rocks_write_buffer_manager_buffer_size(
    rocks_write_buffer_manager_t* manager) {
  return manager->rep->buffer_size();
}
}
