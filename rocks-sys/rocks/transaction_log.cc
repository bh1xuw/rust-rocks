#include "rocks/ctypes.hpp"

using namespace rocksdb;

extern "C" {
void rocks_logfiles_destroy(rocks_logfiles_t* files) { delete files; }

size_t rocks_logfiles_size(rocks_logfiles_t* files) {
  return files->rep.size();
}

void rocks_logfiles_nth_path_name(rocks_logfiles_t* files, size_t nth,
                                  void* s) {
  auto name = files->rep[nth]->PathName();
  rust_string_assign(s, name.data(), name.size());
}

uint64_t rocks_logfiles_nth_log_number(rocks_logfiles_t* files, size_t nth) {
  return files->rep[nth]->LogNumber();
}

int rocks_logfiles_nth_type(rocks_logfiles_t* files, size_t nth) {
  return static_cast<int>(files->rep[nth]->Type());
}

uint64_t rocks_logfiles_nth_start_sequence(rocks_logfiles_t* files,
                                           size_t nth) {
  return files->rep[nth]->StartSequence();
}

uint64_t rocks_logfiles_nth_file_size(rocks_logfiles_t* files, size_t nth) {
  return files->rep[nth]->SizeFileBytes();
}
}
