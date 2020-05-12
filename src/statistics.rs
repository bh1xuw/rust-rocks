//! Analyze the performance of a DB

use std::fmt;
use std::os::raw::c_void;
use std::ptr;

use rocks_sys as ll;

use crate::to_raw::ToRaw;
use crate::{Error, Result};

/// Repr single histogram data item
#[repr(C)]
#[derive(Default, Debug, Clone)]
pub struct HistogramData {
    pub median: f64,
    pub percentile95: f64,
    pub percentile99: f64,
    pub average: f64,
    pub standard_deviation: f64,
    // The following fields were added in newer version of RocksDB
    pub max: f64,
    pub count: u64,
    pub sum: u64,
    pub min: f64,
}

/// Analyze the performance of a db
pub struct Statistics {
    raw: *mut ll::rocks_statistics_t,
}

impl Drop for Statistics {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_statistics_destroy(self.raw);
        }
    }
}

unsafe impl Send for Statistics {}
unsafe impl Sync for Statistics {}

// Clone for shared access
impl Clone for Statistics {
    fn clone(&self) -> Self {
        Statistics {
            raw: unsafe { ll::rocks_statistics_copy(self.raw) },
        }
    }
}

impl ToRaw<ll::rocks_statistics_t> for Statistics {
    fn raw(&self) -> *mut ll::rocks_statistics_t {
        self.raw
    }
}

impl Default for Statistics {
    fn default() -> Self {
        Statistics::new()
    }
}

impl Statistics {
    pub fn new() -> Statistics {
        Statistics {
            raw: unsafe { ll::rocks_statistics_create() },
        }
    }

    pub fn get_ticker_count(&self, ticker: &str) -> u64 {
        unsafe { ll::rocks_statistics_get_ticker_count(self.raw, ticker.as_bytes().as_ptr() as _, ticker.len()) }
    }

    pub fn get_histogram_data(&self, histo: &str) -> HistogramData {
        unsafe {
            let mut data = HistogramData::default();
            ll::rocks_statistics_histogram_data(
                self.raw,
                histo.as_bytes().as_ptr() as _,
                histo.len(),
                &mut data as *mut HistogramData as *mut ll::rocks_histogram_data_t,
            );
            data
        }
    }

    pub fn get_histogram_string(&self, histo: &str) -> String {
        let mut ret = String::new();
        unsafe {
            ll::rocks_statistics_get_histogram_string(
                self.raw,
                histo.as_bytes().as_ptr() as _,
                histo.len(),
                &mut ret as *mut String as *mut _,
            );
        }
        ret
    }

    pub fn get_and_reset_ticker_count(&self, ticker: &str) -> u64 {
        unsafe {
            ll::rocks_statistics_get_and_reset_ticker_count(self.raw, ticker.as_bytes().as_ptr() as _, ticker.len())
        }
    }

    pub fn reset(&self) -> Result<()> {
        let mut status = ptr::null_mut();
        unsafe {
            ll::rocks_statistics_reset(self.raw, &mut status);
        }
        Error::from_ll(status)
    }

    /* NOTE: disable write to Statistics in Rust
    pub fn record_tick(&mut self, ticker_type: Tickers, count: u64) {
        unsafe {
            ll::rocks_statistics_record_tick(self.raw, mem::transmute(ticker_type), count);
        }
    }

    pub fn set_ticker_count(&mut self, ticker_type: Tickers, count: u64) {
        unsafe {
            ll::rocks_statistics_set_ticker_count(self.raw, mem::transmute(ticker_type), count);
        }
    }

    pub fn measure_time(&mut self, histogram_type: Histograms, time: u64) {
        unsafe {
            ll::rocks_statistics_measure_time(self.raw, mem::transmute(histogram_type), time);
        }
    }

    // Override this function to disable particular histogram collection
    pub fn hist_enabled_for_type(&self, type_: Histograms) -> bool {
        unsafe { ll::rocks_statistics_hist_enabled_for_type(self.raw, mem::transmute(type_)) != 0 }
    }
    */
}

impl fmt::Display for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        unsafe {
            ll::rocks_statistics_to_string(self.raw, &mut s as *mut String as *mut c_void);
        }
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::super::rate_limiter::RateLimiter;
    use super::super::rocksdb::*;
    use super::*;

    #[test]
    fn statistics_rate_limiter() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();

        let stat = Statistics::new();

        let db = DB::open(
            Options::default().map_db_options(|db| {
                db.create_if_missing(true)
                    .statistics(Some(stat.clone())) // FIXME: is this the best way?
                    .rate_limiter(Some(RateLimiter::new(
                        4096,    // 4 KiB/s
                        100_000, // 10 ms
                        10,
                    )))
            }),
            &tmp_dir,
        )
        .unwrap();

        assert!(db
            .put(&Default::default(), b"long-key", vec![b'A'; 1024 * 1024].as_ref())
            .is_ok());
        assert!(db.put(&Default::default(), b"a", b"1").is_ok());
        assert!(db.put(&Default::default(), b"b", b"2").is_ok());
        assert!(db.put(&Default::default(), b"c", b"3").is_ok());

        assert!(db.compact_range(&Default::default(), ..).is_ok());

        assert!(db.get(&Default::default(), b"long-key").is_ok());

        println!("st => {}", stat);
        assert!(stat.get_ticker_count("rocksdb.block.cache.bytes.write") > 0);
        // this is the last ticker, since we set up rate limiter to a low value, this must be true
        println!(
            "debug rate limiter: {}",
            stat.get_ticker_count("rocksdb.number.rate_limiter.drains")
        );
        assert!(stat.get_ticker_count("rocksdb.number.rate_limiter.drains") > 0);

        // a multiline string
        assert!(!stat.get_histogram_string("rocksdb.db.write.micros").is_empty());

        stat.get_and_reset_ticker_count("rocksdb.block.cache.bytes.write");
        assert_eq!(stat.get_ticker_count("rocksdb.block.cache.bytes.write"), 0);
    }
}
