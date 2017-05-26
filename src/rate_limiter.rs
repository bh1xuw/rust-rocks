//! RateLimiter object can be shared among RocksDB instances to
//! control write rate of flush and compaction.

use rocks_sys as ll;

use to_raw::ToRaw;

/// RateLimiter object, which can be shared among RocksDB instances to
/// control write rate of flush and compaction.
pub struct RateLimiter {
    raw: *mut ll::rocks_ratelimiter_t,
}

impl Drop for RateLimiter {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_ratelimiter_destroy(self.raw);
        }
    }
}

impl ToRaw<ll::rocks_ratelimiter_t> for RateLimiter {
    fn raw(&self) -> *mut ll::rocks_ratelimiter_t {
        self.raw
    }
}

impl RateLimiter {
    /// Create a RateLimiter object, which can be shared among RocksDB instances to
    /// control write rate of flush and compaction.
    ///
    /// `rate_bytes_per_sec`: this is the only parameter you want to set most of the
    /// time. It controls the total write rate of compaction and flush in bytes per
    /// second. Currently, RocksDB does not enforce rate limit for anything other
    /// than flush and compaction, e.g. write to WAL.
    ///
    /// `refill_period_us`: this controls how often tokens are refilled. For example,
    /// when rate_bytes_per_sec is set to 10MB/s and refill_period_us is set to
    /// 100ms, then 1MB is refilled every 100ms internally. Larger value can lead to
    /// burstier writes while smaller value introduces more CPU overhead.
    /// The default should work for most cases.
    ///
    /// `fairness`: RateLimiter accepts high-pri requests and low-pri requests.
    /// A low-pri request is usually blocked in favor of hi-pri request. Currently,
    /// RocksDB assigns low-pri to request from compaciton and high-pri to request
    /// from flush. Low-pri requests can get blocked if flush requests come in
    /// continuouly. This fairness parameter grants low-pri requests permission by
    /// 1/fairness chance even though high-pri requests exist to avoid starvation.
    pub fn new(rate_bytes_per_sec: i64, refill_period_us: i64, fairness: i32) -> RateLimiter {
        RateLimiter { raw: unsafe { ll::rocks_ratelimiter_create(rate_bytes_per_sec, refill_period_us, fairness) } }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::rocksdb::*;

    use std::time::{Duration, SystemTime};

    #[test]
    fn rate_limiter() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(Options::default()
                          .map_db_options(|db| {
                              db.create_if_missing(true)
                                  .rate_limiter(Some(RateLimiter::new(4096, // 4 KiB/s
                                                                      10_000, // 10 ms
                                                                      10)))
                          })
                          .map_cf_options(|cf| cf.compression(CompressionType::NoCompression)),
                          &tmp_dir)
            .unwrap();

        let now = SystemTime::now();
        assert!(db.put(&Default::default(), b"a", b"1").is_ok());
        assert!(db.compact_range(&Default::default(), ..).is_ok());
        assert!(now.elapsed().unwrap().as_secs() <= 1);


        let now = SystemTime::now();
        // no compression, 8K
        assert!(db.put(&Default::default(), &vec![b'A'; 4 * 1024], &vec![b'B'; 4 * 1024])
                .is_ok());
        assert!(db.compact_range(&Default::default(), ..).is_ok());
        assert!(now.elapsed().unwrap() > Duration::from_secs(1));
    }
}
