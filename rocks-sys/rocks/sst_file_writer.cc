#include "rocksdb/sst_file_writer.h"
#include "rocksdb/comparator.h"

#include "rocks/ctypes.hpp"


using namespace rocksdb;

using std::shared_ptr;

extern "C" {

  rocks_sst_file_writer_t* rocks_sst_file_writer_create_from_c_comparator(
                                                        const rocks_envoptions_t* env_options,
                                                        const rocks_options_t* options,
                                                        const Comparator* comparator,
                                                        rocks_column_family_handle_t* column_family,
                                                        unsigned char invalidate_page_cache) {
    rocks_sst_file_writer_t* result = new rocks_sst_file_writer_t;
    result->rep = new SstFileWriter(
                                    env_options->rep,
                                    options->rep,
                                    comparator,
                                    (column_family != nullptr) ? column_family->rep : nullptr,
                                    invalidate_page_cache != 0);
    return result;
  }

  rocks_sst_file_writer_t* rocks_sst_file_writer_create_from_rust_comparator(
                                                                          const rocks_envoptions_t* env_options,
                                                                          const rocks_options_t* options,
                                                                          const void* comparator_trait_obj,
                                                                          rocks_column_family_handle_t* column_family,
                                                                          unsigned char invalidate_page_cache) {
    rocks_sst_file_writer_t* result = new rocks_sst_file_writer_t;
    result->rep = new SstFileWriter(
                                    env_options->rep,
                                    options->rep,
                                    new rocks_comparator_t { (void *)comparator_trait_obj }, // FIXME: memory leaks
                                    (column_family != nullptr) ? column_family->rep : nullptr,
                                    invalidate_page_cache != 0);
    return result;
  }

  void rocks_sst_file_writer_destroy(rocks_sst_file_writer_t* writer) {
    delete writer;
  }
}
