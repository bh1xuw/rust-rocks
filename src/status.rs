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

use rocks_sys as ll;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Code {
    Ok = 0,
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
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Status {
    pub code: Code,
    pub subcode: SubCode,
    /// string indicating the message of the Status
    pub status: String,
}

impl fmt::Debug for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}({:?}, {:?})", self.code, self.subcode, self.status)
    }
}

impl Status {
    /// Create a success status.
    pub fn new() -> Status {
        Status {
            code: Code::Ok,
            subcode: SubCode::None,
            status: String::new(),
        }
    }

    pub fn is_not_found(&self) -> bool {
        self.code == Code::NotFound
    }

    pub fn from_ll(raw: &ll::rocks_status_t) -> Status {
        unsafe {
            Status {
                code: mem::transmute(raw.code),
                subcode: mem::transmute(raw.sub_code),
                status: {
                    raw.state
                        .as_ref()
                        .and_then(|p| CStr::from_ptr(p).to_str().ok())
                        .map(|s| s.to_owned())
                        .unwrap_or_default()
                },
            }
        }
    }

    // Return a success status.
    // pub fn Ok() -> Status {
    // Status::new()
    // }
    //
    // Return error status of an appropriate type.
    // pub fn NotFound(msg: SubCode) -> Status {
    // Status {
    // code: Code::NotFound,
    // subcode: msg,
    // status: String::new(),
    // }
    // }
    //
    /// Returns true iff the status indicates success.
    pub fn ok(&self) -> bool {
        self.code == Code::Ok
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Status({:?}, {:?}, {})", self.code, self.subcode, self.status)
    }
}
