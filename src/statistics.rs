//! Analyze the performance of a DB

#[repr(C)]
pub struct HistogramData {
    pub median: f64,
    pub percentile95: f64,
    pub percentile99: f64,
    pub average: f64,
    pub standard_deviation: f64,
    pub max: f64,
}

#[repr(C)]
pub enum StatsLevel {
    /// Collect all stats except time inside mutex lock AND time spent on
    /// compression.
    ExceptDetailedTimers,
    /// Collect all stats except the counters requiring to get time inside the
    /// mutex lock.
    ExceptTimeForMutex,
    /// Collect all stats, including measuring duration of mutex operations.
    /// If getting time is expensive on the platform to run, it can
    /// reduce scalability to more threads, especially for writes.
    All,
}

/// Analyze the performance of a db
pub struct Statistics;

impl Statistics {
    pub fn new() -> Statistics {
        unimplemented!()
    }
}
