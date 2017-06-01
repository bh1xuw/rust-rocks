#include "rocksdb/metadata.h"

#include "rocks/ctypes.hpp"

using namespace rocksdb;

extern "C" {
int rocks_livefiles_count(const rocks_livefiles_t* lf) {
  return static_cast<int>(lf->rep.size());
}

size_t rocks_livefiles_size(const rocks_livefiles_t* lf, int index) {
  return lf->rep[index].size;
}

const char* rocks_livefiles_name(const rocks_livefiles_t* lf, int index) {
  return lf->rep[index].name.c_str();
}

const char* rocks_livefiles_column_family_name(const rocks_livefiles_t* lf,
                                               int index) {
  return lf->rep[index].column_family_name.c_str();
}

const char* rocks_livefiles_db_path(const rocks_livefiles_t* lf, int index) {
  return lf->rep[index].db_path.c_str();
}

uint64_t rocks_livefiles_smallest_seqno(const rocks_livefiles_t* lf,
                                        int index) {
  return lf->rep[index].smallest_seqno;
}

uint64_t rocks_livefiles_largest_seqno(const rocks_livefiles_t* lf, int index) {
  return lf->rep[index].largest_seqno;
}

int rocks_livefiles_level(const rocks_livefiles_t* lf, int index) {
  return lf->rep[index].level;
}

const char* rocks_livefiles_smallestkey(const rocks_livefiles_t* lf, int index,
                                        size_t* size) {
  *size = lf->rep[index].smallestkey.size();
  return lf->rep[index].smallestkey.data();
}

const char* rocks_livefiles_largestkey(const rocks_livefiles_t* lf, int index,
                                       size_t* size) {
  *size = lf->rep[index].largestkey.size();
  return lf->rep[index].largestkey.data();
}

unsigned char rocks_livefiles_being_compacted(const rocks_livefiles_t* lf,
                                              int index) {
  return lf->rep[index].being_compacted;
}

extern void rocks_livefiles_destroy(const rocks_livefiles_t* lf) { delete lf; }
}

extern "C" {
uint64_t rocks_column_family_metadata_size(
    const rocks_column_family_metadata_t* meta) {
  return meta->rep.size;
}

size_t rocks_column_family_metadata_file_count(
    const rocks_column_family_metadata_t* meta) {
  return meta->rep.file_count;
}

const char* rocks_column_family_metadata_name(
    const rocks_column_family_metadata_t* meta) {
  return meta->rep.name.c_str();
}

int rocks_column_family_metadata_levels_count(
    const rocks_column_family_metadata_t* meta) {
  return meta->rep.levels.size();
}

int rocks_column_family_metadata_levels_level(
    const rocks_column_family_metadata_t* meta, int level) {
  return meta->rep.levels[level].level;
}

uint64_t rocks_column_family_metadata_levels_size(
    const rocks_column_family_metadata_t* meta, int level) {
  return meta->rep.levels[level].size;
}

int rocks_column_family_metadata_levels_files_count(
    const rocks_column_family_metadata_t* meta, int level) {
  return meta->rep.levels[level].files.size();
}

size_t rocks_column_family_metadata_levels_files_size(
    const rocks_column_family_metadata_t* meta, int level, int file_index) {
  return meta->rep.levels[level].files[file_index].size;
}

const char* rocks_column_family_metadata_levels_files_name(
    const rocks_column_family_metadata_t* meta, int level, int file_index) {
  return meta->rep.levels[level].files[file_index].name.c_str();
}

const char* rocks_column_family_metadata_levels_files_db_path(
    const rocks_column_family_metadata_t* meta, int level, int file_index) {
  return meta->rep.levels[level].files[file_index].db_path.c_str();
}

uint64_t rocks_column_family_metadata_levels_files_smallest_seqno(
    const rocks_column_family_metadata_t* meta, int level, int file_index) {
  return meta->rep.levels[level].files[file_index].smallest_seqno;
}

uint64_t rocks_column_family_metadata_levels_files_largest_seqno(
    const rocks_column_family_metadata_t* meta, int level, int file_index) {
  return meta->rep.levels[level].files[file_index].largest_seqno;
}
const char* rocks_column_family_metadata_levels_files_smallestkey(
    const rocks_column_family_metadata_t* meta, int level, int file_index,
    size_t* size) {
  *size = meta->rep.levels[level].files[file_index].smallestkey.size();
  return meta->rep.levels[level].files[file_index].smallestkey.data();
}

const char* rocks_column_family_metadata_levels_files_largestkey(
    const rocks_column_family_metadata_t* meta, int level, int file_index,
    size_t* size) {
  *size = meta->rep.levels[level].files[file_index].largestkey.size();
  return meta->rep.levels[level].files[file_index].largestkey.data();
}

unsigned char rocks_column_family_metadata_levels_files_being_compacted(
    const rocks_column_family_metadata_t* meta, int level, int file_index) {
  return meta->rep.levels[level].files[file_index].being_compacted;
}

extern void rocks_column_family_metadata_destroy(
    const rocks_column_family_metadata_t* meta) {
  delete meta;
}
}
