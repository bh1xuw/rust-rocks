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

  void rocks_env_set_background_threads(rocks_env_t* env, int n) {
    env->rep->SetBackgroundThreads(n);
  }

  void rocks_env_set_high_priority_background_threads(rocks_env_t* env, int n) {
    env->rep->SetBackgroundThreads(n, Env::HIGH);
  }

  void rocks_env_join_all_threads(rocks_env_t* env) {
    env->rep->WaitForJoin();
  }

  void rocks_env_destroy(rocks_env_t* env) {
    if (!env->is_default) delete env->rep;
    delete env;
  }
}

extern "C" {
  rocks_envoptions_t* rocks_envoptions_create() {
    rocks_envoptions_t* opt = new rocks_envoptions_t;
    return opt;
  }

  void rocks_envoptions_destroy(rocks_envoptions_t* opt) { delete opt; }
}

extern "C" {
  void rocks_logger_destroy(rocks_logger_t *logger) { delete logger; }
}
