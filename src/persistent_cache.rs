//! Persistent cache interface for caching IO pages on a persistent medium.

use std::path::Path;
use std::ptr;
use std::fmt;
use std::str;
use std::slice;

use rocks_sys as ll;

use error::Status;
use env::{Env, Logger};
use to_raw::ToRaw;
use super::Result;

/// Persistent cache interface for caching IO pages on a persistent medium. The
/// cache interface is specifically designed for persistent read cache.
pub struct PersistentCache {
    raw: *mut ll::rocks_persistent_cache_t,
}

impl ToRaw<ll::rocks_persistent_cache_t> for PersistentCache {
    fn raw(&self) -> *mut ll::rocks_persistent_cache_t {
        self.raw
    }
}

impl Clone for PersistentCache {
    /// Duplicated PersistentCache inner shared_ptr
    fn clone(&self) -> Self {
        PersistentCache { raw: unsafe { ll::rocks_persistent_cache_clone(self.raw) } }
    }
}

impl fmt::Debug for PersistentCache {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PersistentCache")
            .field("options", &self.get_printable_options())
            .finish()
    }
}


impl PersistentCache {
    /// Factor method to create a new persistent cache
    pub fn new<P: AsRef<Path>>(
        env: &Env,
        path: P,
        size: u64,
        log: &Logger,
        optimized_for_nvm: bool,
    ) -> Result<PersistentCache> {
        let path_str = path.as_ref().to_str().expect("valid utf8");
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            let raw = ll::rocks_new_persistent_cache(
                env.raw(),
                path_str.as_ptr() as *const _,
                path_str.len(),
                size,
                log.raw(),
                optimized_for_nvm as u8,
                &mut status,
            );
            Status::from_ll(status)
                .map(|()| {
                    PersistentCache {
                        raw: raw,
                    }
                })
        }
    }

    pub fn get_printable_options(&self) -> String {
        unsafe {
            let cxx_string = ll::rocks_persistent_cache_get_printable_options(self.raw);
            let ptr = ll::cxx_string_data(cxx_string) as *const u8;
            let len = ll::cxx_string_size(cxx_string);
            let ret = str::from_utf8_unchecked(slice::from_raw_parts(ptr, len)).into();
            ll::cxx_string_destroy(cxx_string);
            ret
        }
    }
}

#[test]
fn test_persistent_cache() {
    let tmp_dir = ::tempdir::TempDir::new_in("", "rocks").unwrap();
    let logger = Env::default_instance()
        .create_logger(tmp_dir.path().join("test.logfiles"))
        .unwrap();
    // NOTE: from RocksdB, size should be big enough
    let pcache = PersistentCache::new(Env::default_instance(), tmp_dir.path(), 1 << 30, &logger, true).unwrap();

    assert!(format!("{:?}", pcache).contains("is_compressed: 1"));
}
