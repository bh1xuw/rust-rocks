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

// rocks_transaction_log_iterator_t
void rocks_transaction_log_iterator_destory(
    rocks_transaction_log_iterator_t* it) {
  delete it;
}

unsigned char rocks_transaction_log_iterator_valid(
    rocks_transaction_log_iterator_t* it) {
  return it->rep->Valid();
}

void rocks_transaction_log_iterator_next(rocks_transaction_log_iterator_t* it) {
  it->rep->Next();
}

void rocks_transaction_log_iterator_status(rocks_transaction_log_iterator_t* it,
                                           rocks_status_t** status) {
  SaveError(status, it->rep->status());
}

rocks_writebatch_t* rocks_transaction_log_iterator_get_batch(
    rocks_transaction_log_iterator_t* it, uint64_t* seq_no) {
  auto batch = it->rep->GetBatch();
  *seq_no = batch.sequence;
  auto writebatch = new rocks_writebatch_t;
  batch.writeBatchPtr.swap(writebatch->rep);
  return writebatch;
}
}
