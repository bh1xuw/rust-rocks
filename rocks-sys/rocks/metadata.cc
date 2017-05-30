#include "rocksdb/metadata.h"

#include "rocks/ctypes.hpp"


using namespace rocksdb;

extern "C" {
  int rocks_livefiles_count(
                            const rocks_livefiles_t* lf) {
    return static_cast<int>(lf->rep.size());
  }

  const char* rocks_livefiles_name(
                                   const rocks_livefiles_t* lf,
                                   int index) {
    return lf->rep[index].name.c_str();
  }

  const char* rocks_livefiles_column_family_name(
                                                 const rocks_livefiles_t* lf,
                                                 int index) {
    return lf->rep[index].column_family_name.c_str();
  }

  const char* rocks_livefiles_db_path(
                                      const rocks_livefiles_t* lf,
                                      int index) {
    return lf->rep[index].db_path.c_str();
  }

  uint64_t rocks_livefiles_smallest_seqno(
                                          const rocks_livefiles_t* lf,
                                          int index) {
    return lf->rep[index].smallest_seqno;
  }

  uint64_t rocks_livefiles_largest_seqno(
                                          const rocks_livefiles_t* lf,
                                          int index) {
    return lf->rep[index].largest_seqno;
  }


  int rocks_livefiles_level(
                            const rocks_livefiles_t* lf,
                            int index) {
    return lf->rep[index].level;
  }


  size_t rocks_livefiles_size(
                                const rocks_livefiles_t* lf,
                                int index) {
    return lf->rep[index].size;
  }

  const char* rocks_livefiles_smallestkey(
                                            const rocks_livefiles_t* lf,
                                            int index,
                                            size_t* size) {
    *size = lf->rep[index].smallestkey.size();
    return lf->rep[index].smallestkey.data();
  }

  const char* rocks_livefiles_largestkey(
                                           const rocks_livefiles_t* lf,
                                           int index,
                                           size_t* size) {
    *size = lf->rep[index].largestkey.size();
    return lf->rep[index].largestkey.data();
  }

  unsigned char rocks_livefiles_being_compacted(
                                                const rocks_livefiles_t* lf,
                                                int index) {
    return lf->rep[index].being_compacted;
  }


  extern void rocks_livefiles_destroy(
                                        const rocks_livefiles_t* lf) {
    delete lf;
  }
}
