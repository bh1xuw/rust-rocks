//! Debug helper functions

use std::fmt;
use std::slice;
use std::mem;
use std::ops;
use std::iter;
use std::marker::PhantomData;

use rocks_sys as ll;

use to_raw::{FromRaw, ToRaw};
use types::SequenceNumber;


// Value types encoded as the last component of internal keys.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ValueType {
    Deletion = 0x0,
    Value = 0x1,
    Merge = 0x2,
    LogData = 0x3, // WAL only.
    ColumnFamilyDeletion = 0x4, // WAL only.
    ColumnFamilyValue = 0x5, // WAL only.
    ColumnFamilyMerge = 0x6, // WAL only.
    SingleDeletion = 0x7,
    ColumnFamilySingleDeletion = 0x8, // WAL only.
    BeginPrepareXID = 0x9, // WAL only.
    EndPrepareXID = 0xA, // WAL only.
    CommitXID = 0xB, // WAL only.
    RollbackXID = 0xC, // WAL only.
    Noop = 0xD, // WAL only.
    ColumnFamilyRangeDeletion = 0xE, // WAL only.
    RangeDeletion = 0xF, // meta block
}

// Data associated with a particular version of a key. A database may internally
// store multiple versions of a same user key due to snapshots, compaction not
// happening yet, etc.
pub struct KeyVersion {
    raw: ll::rocks_key_version_t,
}

impl fmt::Debug for KeyVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("KeyVersion")
            .field("user_key", &self.user_key())
            .field("sequence", &self.sequence())
            .field("type", &self.value_type())
            .finish()
    }
}

impl ToRaw<ll::rocks_key_version_t> for KeyVersion {
    fn raw(&self) -> *mut ll::rocks_key_version_t {
        self as *const KeyVersion as *mut ll::rocks_key_version_t
    }
}

impl<'a> FromRaw<ll::rocks_key_version_t> for &'a KeyVersion {
    unsafe fn from_ll(raw: *mut ll::rocks_key_version_t) -> Self {
        &*(raw as *mut KeyVersion)
    }
}

impl KeyVersion {
    pub fn user_key(&self) -> &[u8] {
        unsafe {
            let mut keylen = 0;
            let key_ptr = ll::rocks_key_version_user_key(self.raw(), &mut keylen);
            slice::from_raw_parts(key_ptr as *const u8, keylen)
        }
    }

    pub fn value(&self) -> &[u8] {
        unsafe {
            let mut vallen = 0;
            let val_ptr = ll::rocks_key_version_value(self.raw(), &mut vallen);
            slice::from_raw_parts(val_ptr as *const u8, vallen)
        }
    }

    pub fn sequence(&self) -> SequenceNumber {
        unsafe { ll::rocks_key_version_sequence_numer(self.raw()).into() }
    }

    pub fn value_type(&self) -> ValueType {
        unsafe { mem::transmute(ll::rocks_key_version_type(self.raw())) }
    }
}

pub struct KeyVersionVec<'a> {
    raw: *mut ll::rocks_key_version_collection_t,
    _marker: PhantomData<&'a ()>,
}

impl<'a> fmt::Debug for KeyVersionVec<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "KeyVersionVec {{ size: {} }}", self.len())
    }
}

impl<'a> Drop for KeyVersionVec<'a> {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_key_version_collection_destroy(self.raw);
        }
    }
}

impl<'a> FromRaw<ll::rocks_key_version_collection_t> for KeyVersionVec<'a> {
    unsafe fn from_ll(raw: *mut ll::rocks_key_version_collection_t) -> Self {
        KeyVersionVec {
            raw: raw,
            _marker: PhantomData,
        }
    }
}

impl<'a> KeyVersionVec<'a> {
    pub fn len(&self) -> usize {
        unsafe { ll::rocks_key_version_collection_size(self.raw) }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> KeyVersionVecIter {
        KeyVersionVecIter {
            vec: self,
            idx: 0,
        }
    }
}

impl<'a> iter::IntoIterator for &'a KeyVersionVec<'a> {
    type Item = &'a KeyVersion;
    type IntoIter = KeyVersionVecIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> ops::Index<usize> for KeyVersionVec<'a> {
    type Output = KeyVersion;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len());
        unsafe { FromRaw::from_ll(ll::rocks_key_version_collection_nth(self.raw, index)) }
    }
}

pub struct KeyVersionVecIter<'a> {
    vec: &'a KeyVersionVec<'a>,
    idx: usize,
}

impl<'a> iter::Iterator for KeyVersionVecIter<'a> {
    type Item = &'a KeyVersion;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.vec.len() {
            None
        } else {
            unsafe {
                let p = ll::rocks_key_version_collection_nth(self.vec.raw, self.idx);
                self.idx += 1;
                Some(FromRaw::from_ll(p))
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use std::iter;
    use super::super::rocksdb::*;

    #[test]
    fn key_version() {
        let tmp_dir = ::tempdir::TempDir::new_in("", "rocks").unwrap();
        let db = DB::open(
            Options::default()
                .map_db_options(|db| db.create_if_missing(true))
                .map_cf_options(|cf| cf.disable_auto_compactions(true)),
            &tmp_dir,
        ).unwrap();

        for i in 0..100 {
            let key = format!("k{}", i % 20);
            let val = format!("v{}", i * i);
            let value: String = iter::repeat(val).take(i * i).collect::<Vec<_>>().concat();

            db.put(WriteOptions::default_instance(), key.as_bytes(), value.as_bytes())
                .unwrap();

            if i % 13 == 0 {
                db.delete(WriteOptions::default_instance(), key.as_bytes())
                    .unwrap();
            }
        }

        let vers = db.get_all_key_versions(b"\x00", b"\xff");
        assert!(vers.is_ok());
        let vers = vers.unwrap();
        // assert_eq!(vers.len(), 100);
        for i in 0..99 {
            assert!(vers[i].user_key().len() > 0);
        }

        for v in &vers {
            assert!(true, "iterator works");
            return;
        }
        assert!(false);
    }
}
