//! Define all public custom types here.

use std::convert::From;
use std::mem;
use std::ops::Deref;
use std::str;

/// Represents a sequence number in a WAL file.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SequenceNumber(pub u64);

/// 0 is always committed
pub const MIN_UNCOMMITTED_SEQ: SequenceNumber = SequenceNumber(1);

impl ::std::fmt::Display for SequenceNumber {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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

/// User-oriented representation of internal key types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EntryType {
    Put = 0x1,
    Delete = 0x0,
    SingleDelete = 0x7,
    Merge = 0x2,
    RangeDeletion = 0xf,
    BlobIndex = 0x11,
    Other,
}

impl EntryType {
    pub fn from_u8(val: u8) -> EntryType {
        use EntryType::*;

        match val {
            0x1 => Put,
            0x0 => Delete,
            0x7 => SingleDelete,
            0x2 => Merge,
            0xf => RangeDeletion,
            0x11 => BlobIndex,
            _ => Other,
        }
    }
}

/// <user key, sequence number, and entry type> tuple.
pub struct FullKey<'a> {
    user_key: &'a [u8],
    sequence: SequenceNumber,
    entry_type: EntryType,
}

impl ::std::fmt::Debug for FullKey<'_> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("FullKey")
            .field("user_key", &str::from_utf8(self.user_key))
            .field("sequence", &self.sequence)
            .field("type", &self.entry_type)
            .finish()
    }
}

impl FullKey<'_> {
    pub fn new<'b>(u: &'b [u8], seq: SequenceNumber, t: EntryType) -> FullKey<'b> {
        FullKey {
            user_key: u,
            sequence: seq,
            entry_type: t,
        }
    }

    /// Parse slice representing internal key to FullKey
    /// Parsed FullKey is valid for as long as the memory pointed to by
    /// internal_key is alive.
    pub fn parse<'b>(internal_key: &'b [u8]) -> Option<FullKey<'b>> {
        // via dbformat.h
        let n = internal_key.len();
        if n < 8 {
            return None;
        }
        let mut raw_num = [0u8; 8];
        raw_num.copy_from_slice(&internal_key[n - 8..]);
        let num: u64 = unsafe { mem::transmute(raw_num) };
        println!("num ={}", num);
        let c = (num & 0xff) as u8;
        let seq = num >> 8;
        let typ = EntryType::from_u8(c);
        let user_key = &internal_key[..n - 8];

        Some(FullKey::new(user_key, SequenceNumber(seq), typ))
    }
}
