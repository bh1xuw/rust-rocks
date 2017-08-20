#include "rocksdb/db_dump_tool.h"

#include "rocks/ctypes.hpp"

using namespace rocksdb;

extern "C" {
rocks_dump_options_t* rocks_dump_options_create() { return new rocks_dump_options_t; }

void rocks_dump_options_destroy(rocks_dump_options_t* options) { delete options; }

void rocks_dump_options_set_db_path(rocks_dump_options_t* opt, const char* path, const size_t path_len) {
  opt->rep.db_path.assign(path, path_len);
}

void rocks_dump_options_set_dump_location(rocks_dump_options_t* opt, const char* path, const size_t path_len) {
  opt->rep.dump_location.assign(path, path_len);
}

void rocks_dump_options_set_anonymous(rocks_dump_options_t* opt, unsigned char v) { opt->rep.anonymous = v; }

unsigned char rocks_db_dump_tool_run(rocks_dump_options_t* dump_options, rocks_options_t* options) {
  auto tool = DbDumpTool();
  return tool.Run(dump_options->rep, options->rep);
}

rocks_undump_options_t* rocks_undump_options_create() { return new rocks_undump_options_t; }

void rocks_undump_options_destroy(rocks_undump_options_t* options) { delete options; }

void rocks_undump_options_set_db_path(rocks_undump_options_t* opt, const char* path, const size_t path_len) {
  opt->rep.db_path.assign(path, path_len);
}

void rocks_undump_options_set_dump_location(rocks_undump_options_t* opt, const char* path, const size_t path_len) {
  opt->rep.dump_location.assign(path, path_len);
}

void rocks_undump_options_set_compact_db(rocks_undump_options_t* opt, unsigned char v) { opt->rep.compact_db = v; }

unsigned char rocks_db_undump_tool_run(rocks_undump_options_t* undump_options, rocks_options_t* options) {
  auto tool = DbUndumpTool();
  return tool.Run(undump_options->rep, options->rep);
}
}
