
pub struct RateLimiter;


/// Create a RateLimiter object, which can be shared among RocksDB instances to
/// control write rate of flush and compaction.
/// @rate_bytes_per_sec: this is the only parameter you want to set most of the
/// time. It controls the total write rate of compaction and flush in bytes per
/// second. Currently, RocksDB does not enforce rate limit for anything other
/// than flush and compaction, e.g. write to WAL.
/// @refill_period_us: this controls how often tokens are refilled. For example,
/// when rate_bytes_per_sec is set to 10MB/s and refill_period_us is set to
/// 100ms, then 1MB is refilled every 100ms internally. Larger value can lead to
/// burstier writes while smaller value introduces more CPU overhead.
/// The default should work for most cases.
/// @fairness: RateLimiter accepts high-pri requests and low-pri requests.
/// A low-pri request is usually blocked in favor of hi-pri request. Currently,
/// RocksDB assigns low-pri to request from compaciton and high-pri to request
/// from flush. Low-pri requests can get blocked if flush requests come in
/// continuouly. This fairness parameter grants low-pri requests permission by
/// 1/fairness chance even though high-pri requests exist to avoid starvation.
impl RateLimiter {
    pub fn new(rate_bytes_per_sec: i64, refill_period_us: i64, fairness: i32) -> RateLimiter {
        // extern RateLimiter* NewGenericRateLimiter(
        // int64_t rate_bytes_per_sec,
        // int64_t refill_period_us = 100 * 1000,
        // int32_t fairness = 10);
        //
        unimplemented!()
    }
}
