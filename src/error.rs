//! A Status encapsulates the result of an operation.
//!
//! It may indicate success,
//! or it may indicate an error with an associated error message.
//!
//! Multiple threads can invoke const methods on a Status without
//! external synchronization, but if any of the threads may call a
//! non-const method, all threads accessing the same Status must use
//! external synchronization.

use std::fmt;
use std::mem;
use std::ffi::CStr;
use std::str;

use rocks_sys as ll;

use crate::to_raw::{FromRaw, ToRaw};

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Code {
    _Ok = 0, // will never be available
    NotFound = 1,
    Corruption = 2,
    NotSupported = 3,
    InvalidArgument = 4,
    IOError = 5,
    MergeInProgress = 6,
    Incomplete = 7,
    ShutdownInProgress = 8,
    TimedOut = 9,
    Aborted = 10,
    Busy = 11,
    Expired = 12,
    TryAgain = 13,
    CompactionTooLarge = 14,
    ColumnFamilyDropped = 15,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SubCode {
    None = 0,
    MutexTimeout = 1,
    LockTimeout = 2,
    LockLimit = 3,
    NoSpace = 4,
    Deadlock = 5,
    StaleFile = 6,
    MemoryLimit = 7,
    SpaceLimit = 8,
    PathNotFound = 9,
    MergeOperandsInsufficientCapacity = 10,
    ManualCompactionPaused = 11,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Status {
    raw: *mut ll::rocks_status_t,
}

impl ToRaw<ll::rocks_status_t> for Status {
    fn raw(&self) -> *mut ll::rocks_status_t {
        self.raw
    }
}

impl FromRaw<ll::rocks_status_t> for Result<(), Status> {
    unsafe fn from_ll(raw: *mut ll::rocks_status_t) -> Result<(), Status> {
        if raw.is_null() || ll::rocks_status_code(raw) == 0 {
            Ok(())
        } else {
            Err(Status { raw: raw })
        }
    }
}

impl Drop for Status {
    fn drop(&mut self) {
        unsafe { ll::rocks_status_destroy(self.raw) }
    }
}

impl Status {
    pub fn with_message(msg: &'static str) -> Status {
        let code = Code::InvalidArgument;
        assert!(code != Code::_Ok, "Can't create a Ok status in Rust");
        unsafe {
            let ccode = mem::transmute(code);
            Status { raw: ll::rocks_status_create_with_code_and_msg(ccode, msg.as_ptr() as *const _, msg.len()) }
        }
    }

    pub fn is_not_found(&self) -> bool {
        self.code() == Code::NotFound
    }

    pub fn code(&self) -> Code {
        unsafe { mem::transmute(ll::rocks_status_code(self.raw)) }
    }

    pub fn subcode(&self) -> SubCode {
        unsafe { mem::transmute(ll::rocks_status_subcode(self.raw)) }
    }

    /// string indicating the message of the Status
    pub fn state(&self) -> &str {
        unsafe {
            let ptr = ll::rocks_status_get_state(self.raw);
            ptr.as_ref()
                .and_then(|s| CStr::from_ptr(s).to_str().ok())
                .unwrap_or("")
        }
    }

    pub fn from_ll(raw: *mut ll::rocks_status_t) -> Result<(), Status> {
        if raw.is_null() || unsafe { ll::rocks_status_code(raw) } == 0 {
            Ok(())
        } else {
            Err(Status { raw: raw })
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Status({:?}, {:?}, {})", self.code(), self.subcode(), self.state())
    }
}

impl fmt::Debug for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}({:?}, {:?})", self.code(), self.subcode(), self.state())
    }
}
