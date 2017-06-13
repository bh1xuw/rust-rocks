//! Define all public custom types here.

use std::ops::Deref;
use std::convert::From;

/// Represents a sequence number in a WAL file.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SequenceNumber(pub u64);

impl From<u64> for SequenceNumber {
    fn from(x: u64) -> SequenceNumber {
        SequenceNumber(x)
    }
}

impl From<SequenceNumber> for u64 {
    fn from(SequenceNumber(x): SequenceNumber) -> u64 {
        x
    }
}

impl Deref for SequenceNumber {
    type Target = u64;

    fn deref(&self) -> &u64 {
        &self.0
    }
}
