#include "rocksdb/env.h"

#include "rocks/ctypes.hpp"

using namespace rocksdb;

using std::shared_ptr;

extern "C" {
rocks_env_t* rocks_create_default_env() {
  rocks_env_t* result = new rocks_env_t;
  result->rep = Env::Default();
  result->is_default = true;
  return result;
}

rocks_env_t* rocks_create_mem_env() {
  rocks_env_t* result = new rocks_env_t;
  result->rep = rocksdb::NewMemEnv(Env::Default());
  result->is_default = false;
  return result;
}

rocks_env_t* rocks_create_timed_env() {
  rocks_env_t* result = new rocks_env_t;
  result->rep = rocksdb::NewTimedEnv(Env::Default());
  result->is_default = false;
  return result;
}

void rocks_env_destroy(rocks_env_t* env) {
  if (!env->is_default) delete env->rep;
  delete env;
}

void rocks_env_set_background_threads(rocks_env_t* env, int n) { env->rep->SetBackgroundThreads(n); }

void rocks_env_set_high_priority_background_threads(rocks_env_t* env, int n) {
  env->rep->SetBackgroundThreads(n, Env::HIGH);
}

void rocks_env_join_all_threads(rocks_env_t* env) { env->rep->WaitForJoin(); }

unsigned int rocks_env_get_thread_pool_queue_len(rocks_env_t* env, int pri) {
  return env->rep->GetThreadPoolQueueLen(static_cast<Env::Priority>(pri));
}

rocks_logger_t* rocks_env_new_logger(rocks_env_t* env, const char* name_ptr, size_t name_len, rocks_status_t** status) {
  auto logger = new rocks_logger_t;
  auto st = env->rep->NewLogger(std::string(name_ptr, name_len), &logger->rep);
  if (SaveError(status, std::move(st))) {
    delete logger;
    return nullptr;
  } else {
    return logger;
  }
}

uint64_t rocks_env_now_micros(rocks_env_t* env) { return env->rep->NowMicros(); }

uint64_t rocks_env_now_nanos(rocks_env_t* env) { return env->rep->NowNanos(); }

void rocks_env_sleep_for_microseconds(rocks_env_t* env, int32_t micros) {
  return env->rep->SleepForMicroseconds(micros);
}

void rocks_env_get_host_name(rocks_env_t* env, char* name, uint64_t len, rocks_status_t** status) {
  SaveError(status, env->rep->GetHostName(name, len));
}

int64_t rocks_env_get_current_time(rocks_env_t* env, rocks_status_t** status) {
  int64_t unix_time;
  if (SaveError(status, env->rep->GetCurrentTime(&unix_time))) {
    return 0;
  } else {
    return unix_time;
  }
}

// needs destroy
cxx_string_t* rocks_env_time_to_string(rocks_env_t* env, uint64_t time) {
  auto st = new std::string(env->rep->TimeToString(time));
  return reinterpret_cast<cxx_string_t*>(st);
}

int rocks_env_get_background_threads(rocks_env_t* env, int pri) {
  auto priority = static_cast<Env::Priority>(pri);
  return env->rep->GetBackgroundThreads(priority);
}

void rocks_env_inc_background_threads_if_needed(rocks_env_t* env, int number, int pri) {
  env->rep->IncBackgroundThreadsIfNeeded(number, static_cast<Env::Priority>(pri));
}

void rocks_env_lower_thread_pool_io_priority(rocks_env_t* env, int pool) {
  env->rep->LowerThreadPoolIOPriority(static_cast<Env::Priority>(pool));
}

uint64_t rocks_env_get_thread_id(rocks_env_t* env) { return env->rep->GetThreadID(); }
}

extern "C" {
rocks_envoptions_t* rocks_envoptions_create() {
  rocks_envoptions_t* opt = new rocks_envoptions_t;
  return opt;
}

void rocks_envoptions_destroy(rocks_envoptions_t* opt) { delete opt; }

void rocks_envoptions_set_use_mmap_reads(rocks_envoptions_t* opt, unsigned char val) { opt->rep.use_mmap_reads = val; }
void rocks_envoptions_set_use_mmap_writes(rocks_envoptions_t* opt, unsigned char val) {
  opt->rep.use_mmap_writes = val;
}

void rocks_envoptions_set_use_direct_reads(rocks_envoptions_t* opt, unsigned char val) {
  opt->rep.use_direct_reads = val;
}
void rocks_envoptions_set_use_direct_writes(rocks_envoptions_t* opt, unsigned char val) {
  opt->rep.use_direct_writes = val;
}
void rocks_envoptions_set_allow_fallocate(rocks_envoptions_t* opt, unsigned char val) {
  opt->rep.allow_fallocate = val;
}
// FIXME: bad name?
void rocks_envoptions_set_fd_cloexec(rocks_envoptions_t* opt, unsigned char val) { opt->rep.set_fd_cloexec = val; }
void rocks_envoptions_set_bytes_per_sync(rocks_envoptions_t* opt, uint64_t val) { opt->rep.bytes_per_sync = val; }
void rocks_envoptions_set_fallocate_with_keep_size(rocks_envoptions_t* opt, unsigned char val) {
  opt->rep.fallocate_with_keep_size = val;
}
void rocks_envoptions_set_compaction_readahead_size(rocks_envoptions_t* opt, size_t val) {
  ;
  opt->rep.compaction_readahead_size = val;
}
void rocks_envoptions_set_random_access_max_buffer_size(rocks_envoptions_t* opt, size_t val) {
  ;
  opt->rep.random_access_max_buffer_size = val;
}
void rocks_envoptions_set_writable_file_max_buffer_size(rocks_envoptions_t* opt, size_t val) {
  opt->rep.writable_file_max_buffer_size = val;
}
/*
void rocks_envoptions_set_rate_limiter(rocks_envoptions_t* opt, ....) {

}
*/
}

extern "C" {
void rocks_logger_destroy(rocks_logger_t* logger) { delete logger; }

void rocks_logger_log(rocks_logger_t* logger, int log_level, const char* msg_ptr, size_t msg_len) {
  if (logger->rep) {
    auto msg = std::string(msg_ptr, msg_len);
    va_list ap;
    logger->rep->Logv(static_cast<InfoLogLevel>(log_level), msg.c_str(), ap);
  }
}

void rocks_logger_flush(rocks_logger_t* logger) {
  if (logger->rep) {
    logger->rep->Flush();
  }
}

void rocks_logger_set_log_level(rocks_logger_t* logger, int log_level) {
  if (logger->rep) {
    logger->rep->SetInfoLogLevel(static_cast<InfoLogLevel>(log_level));
  }
}

int rocks_logger_get_log_level(rocks_logger_t* logger) {
  if (logger->rep) {
    return static_cast<int>(logger->rep->GetInfoLogLevel());
  }
  return 0;
}
}
