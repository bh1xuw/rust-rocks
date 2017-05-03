
use std::path::Path;

use env::Env;
use env::Logger;
use status::Status;

/// SstFileManager is used to track SST files in the DB and control there
/// deletion rate.
/// All SstFileManager public functions are thread-safe.
pub struct SstFileManager;


impl SstFileManager {
    pub fn new(env: Env,
               info_log: Option<Logger>,
               trash_dir: &Path,
               rate_bytes_per_sec: i64,
               delete_existing_trash: bool)
               -> Result<SstFileManager, Status> {
        unimplemented!()
    }
}

// extern SstFileManager* NewSstFileManager(
// Env* env, std::shared_ptr<Logger> info_log = nullptr,
// std::string trash_dir = "", int64_t rate_bytes_per_sec = 0,
// bool delete_existing_trash = true, Status* status = nullptr);
//
