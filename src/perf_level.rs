//! Config about how much perf stats to collect

use std::mem;

use rocks_sys as ll;

/// How much perf stats to collect. Affects [`perf_context`] and [`iostats_context`].
///
/// [`perf_context`]: ../../rocks/perf_context/index.html
/// [`iostats_context`]: ../../rocks/iostats_context/index.html
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PerfLevel {
    /// unknown setting
    Uninitialized = 0,
    /// disable perf stats
    Disable = 1,
    /// enable only count stats
    EnableCount = 2,
    /// Other than count stats, also enable time
    /// stats except for mutexes
    EnableTimeExceptForMutex = 3,
    /// enable count and time stats
    EnableTime = 4,
}



/// set the perf stats level for current thread
pub fn set_perf_level(level: PerfLevel) {
    unsafe {
        ll::rocks_set_perf_level(mem::transmute(level));
    }
}


/// get current perf stats level for current thread
pub fn get_perf_level() -> PerfLevel {
    unsafe { mem::transmute(ll::rocks_get_perf_level()) }
}


#[test]
fn test_perf_level() {
    set_perf_level(PerfLevel::Disable);
    assert_eq!(get_perf_level(), PerfLevel::Disable);

    set_perf_level(PerfLevel::EnableTimeExceptForMutex);
    assert_eq!(get_perf_level(), PerfLevel::EnableTimeExceptForMutex);
}
