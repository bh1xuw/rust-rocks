//! An iterator yields a sequence of key/value pairs from a source.

use std::fmt;
use std::iter;
use std::marker::PhantomData;
use std::mem;
use std::os::raw::c_void;
use std::slice;

use rocks_sys as ll;

use crate::error::Status;
use crate::to_raw::FromRaw;
use crate::Result;

/// An iterator yields a sequence of key/value pairs from a source.
///
/// Multiple threads can invoke const methods on an Iterator without
/// external synchronization, but if any of the threads may call a
/// non-const method, all threads accessing the same Iterator must use
/// external synchronization.
pub struct Iterator<'a> {
    raw: *mut ll::rocks_iterator_t,
    _marker: PhantomData<&'a ()>,
}

unsafe impl<'a> Send for Iterator<'a> {}
// unsafe impl Sync for Iterator {}

impl<'a> fmt::Debug for Iterator<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Iterator(")?;
        if self.is_valid() {
            write!(f, "key={:?})", String::from_utf8_lossy(self.key()))
        } else {
            write!(f, "INVALID)")
        }
    }
}

impl<'a> Drop for Iterator<'a> {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_iter_destroy(self.raw);
        }
    }
}

impl<'a> FromRaw<ll::rocks_iterator_t> for Iterator<'a> {
    unsafe fn from_ll(raw: *mut ll::rocks_iterator_t) -> Self {
        Iterator {
            raw: raw,
            _marker: PhantomData,
        }
    }
}

impl<'a> Iterator<'a> {
    /// An iterator is either positioned at a key/value pair, or
    /// not valid.  This method returns true iff the iterator is valid.
    pub fn is_valid(&self) -> bool {
        unsafe { ll::rocks_iter_valid(self.raw) != 0 }
    }

    /// Position at the first key in the source.  The iterator `is_valid()`
    /// after this call iff the source is not empty.
    pub fn seek_to_first(&mut self) {
        unsafe {
            ll::rocks_iter_seek_to_first(self.raw);
        }
    }

    /// Position at the last key in the source.  The iterator
    /// `is_valid()` after this call iff the source is not empty.
    pub fn seek_to_last(&mut self) {
        unsafe {
            ll::rocks_iter_seek_to_last(self.raw);
        }
    }

    /// Position at the first key in the source that at or past target
    /// The iterator `is_valid()` after this call iff the source contains
    /// an entry that comes at or past target.
    pub fn seek(&mut self, target: &[u8]) {
        unsafe {
            ll::rocks_iter_seek(self.raw, target.as_ptr() as _, target.len());
        }
    }

    /// Position at the last key in the source that at or before target
    /// The iterator `is_valid()` after this call iff the source contains
    /// an entry that comes at or before target.
    pub fn seek_for_prev(&mut self, target: &[u8]) {
        unsafe {
            ll::rocks_iter_seek_for_prev(self.raw, target.as_ptr() as _, target.len());
        }
    }

    /// Moves to the next entry in the source.  After this call, `is_valid()` is
    /// true iff the iterator was not positioned at the last entry in the source.
    ///
    /// REQUIRES: `is_valid()`
    pub fn next(&mut self) {
        unsafe {
            ll::rocks_iter_next(self.raw);
        }
    }

    /// Moves to the previous entry in the source.  After this call, `is_valid()` is
    /// true iff the iterator was not positioned at the first entry in source.
    ///
    /// REQUIRES: `is_valid()`
    pub fn prev(&mut self) {
        unsafe {
            ll::rocks_iter_prev(self.raw);
        }
    }

    /// Return the key for the current entry.  The underlying storage for
    /// the returned slice is valid only until the next modification of
    /// the iterator.
    ///
    /// REQUIRES: `is_valid()`
    pub fn key(&self) -> &[u8] {
        unsafe {
            let mut len = 0;
            let ptr = ll::rocks_iter_key(self.raw, &mut len);
            slice::from_raw_parts(ptr as _, len)
        }
    }

    /// Return the value for the current entry.  The underlying storage for
    /// the returned slice is valid only until the next modification of
    /// the iterator.
    ///
    /// REQUIRES: `!AtEnd() && !AtStart()`
    pub fn value(&self) -> &[u8] {
        unsafe {
            let mut len = 0;
            let ptr = ll::rocks_iter_value(self.raw, &mut len);
            slice::from_raw_parts(ptr as _, len)
        }
    }

    /// If an error has occurred, return it.  Else return an ok status.
    /// If non-blocking IO is requested and this operation cannot be
    /// satisfied without doing some IO, then this returns `Status::Incomplete()`.
    pub fn status(&self) -> Result<()> {
        unsafe {
            let mut status = mem::zeroed();
            ll::rocks_iter_get_status(self.raw, &mut status);
            Status::from_ll(status)
        }
    }

    /// Property `"rocksdb.iterator.is-key-pinned"`:
    ///
    /// - If returning "1", this means that the Slice returned by key() is valid as long as the
    ///   iterator is not deleted.
    /// - It is guaranteed to always return "1" if
    ///   - Iterator created with `ReadOptions::pin_data = true`
    ///   - DB tables were created with `BlockBasedTableOptions::use_delta_encoding = false`.
    ///
    /// Property `"rocksdb.iterator.super-version-number"`:
    ///
    /// - LSM version used by the iterator. The same format as DB Property
    /// - `kCurrentSuperVersionNumber`. See its comment for more information.
    pub fn get_property(&self, property: &str) -> Result<String> {
        unsafe {
            let mut ret = String::new();
            let mut status = mem::zeroed();
            ll::rocks_iter_get_property(
                self.raw,
                property.as_bytes().as_ptr() as *const _,
                property.len(),
                &mut ret as *mut String as *mut c_void,
                &mut status,
            );
            // FIXME: rocksdb return error string in get_property
            Status::from_ll(status).map(|_| ret)
        }
    }

    // FIXME: leaks value?
    // TODO: failes under empty db
    /// Consume and make a rust style iterator
    pub fn into_iter(mut self) -> IntoIter<'a> {
        if !self.is_valid() {
            self.seek_to_first();
        }
        assert_eq!(
            self.get_property("rocksdb.iterator.is-key-pinned"),
            Ok("1".to_owned()),
            "key is not pinned!"
        );
        IntoIter { inner: self }
    }

    /// consume and make a reversed rust style iterator
    pub fn into_rev_iter(mut self) -> IntoRevIter<'a> {
        if !self.is_valid() {
            self.seek_to_last();
        }
        assert_eq!(
            self.get_property("rocksdb.iterator.is-key-pinned"),
            Ok("1".to_owned()),
            "key is not pinned!"
        );
        IntoRevIter { inner: self }
    }
}

impl<'a> iter::IntoIterator for Iterator<'a> {
    type Item = (&'a [u8], &'a [u8]);
    type IntoIter = IntoIter<'a>;

    fn into_iter(mut self) -> Self::IntoIter {
        if !self.is_valid() {
            self.seek_to_first();
        }
        // FIXME: is-key-pinned really used?
        /*assert_eq!(
            self.get_property("rocksdb.iterator.is-key-pinned"),
            Ok("1".to_owned()),
            "key is not pinned!"
        );*/
        IntoIter { inner: self }
    }
}

/// Wraps into a rust-style Iterator
pub struct IntoIter<'a> {
    inner: Iterator<'a>,
}

impl<'a> IntoIter<'a> {
    pub fn into_inner(self) -> Iterator<'a> {
        self.inner
    }
}

impl<'a> iter::Iterator for IntoIter<'a> {
    type Item = (&'a [u8], &'a [u8]);

    // FIXME: is it dangerous if data is un-pinned?
    fn next(&mut self) -> Option<(&'a [u8], &'a [u8])> {
        if self.inner.is_valid() {
            // let ret = Some((self.inner.key(), self.inner.value()));
            let k = unsafe {
                let mut len = 0;
                let ptr = ll::rocks_iter_key(self.inner.raw, &mut len);
                slice::from_raw_parts(ptr as _, len)
            };
            let v = unsafe {
                let mut len = 0;
                let ptr = ll::rocks_iter_value(self.inner.raw, &mut len);
                slice::from_raw_parts(ptr as _, len)
            };
            self.inner.next();
            Some((k, v))
        } else {
            None
        }
    }
}

/// Wraps reverse iteration into a rust-style Iterator
pub struct IntoRevIter<'a> {
    inner: Iterator<'a>,
}

impl<'a> IntoRevIter<'a> {
    pub fn into_inner(self) -> Iterator<'a> {
        self.inner
    }
}

impl<'a> iter::Iterator for IntoRevIter<'a> {
    type Item = (&'a [u8], &'a [u8]);

    // FIXME: is it dangerous if data is un-pinned?
    fn next(&mut self) -> Option<(&'a [u8], &'a [u8])> {
        if self.inner.is_valid() {
            // let ret = Some((self.inner.key(), self.inner.value()));
            let k = unsafe {
                let mut len = 0;
                let ptr = ll::rocks_iter_key(self.inner.raw, &mut len);
                slice::from_raw_parts(ptr as _, len)
            };
            let v = unsafe {
                let mut len = 0;
                let ptr = ll::rocks_iter_value(self.inner.raw, &mut len);
                slice::from_raw_parts(ptr as _, len)
            };
            self.inner.prev();
            Some((k, v))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::rocksdb::*;

    #[test]
    fn iterator() {
        use tempdir::TempDir;
        let tmp_dir = TempDir::new_in(".", "rocks").unwrap();
        let opt = Options::default()
            .map_db_options(|db| db.create_if_missing(true))
            .map_cf_options(|cf| {
                cf.table_factory_block_based(BlockBasedTableOptions::default().use_delta_encoding(false))
            });
        let db = DB::open(opt, tmp_dir.path()).unwrap();
        let mut batch = WriteBatch::new();

        batch
            .put(b"key1", b"BYasdf1CQ")
            .put(b"key2", b"BYasdf1CQ")
            .put(b"key3", b"BYasdf1CQ")
            .put(b"key4", b"BY1dfsgCQ")
            .put(b"key5", b"BY1ghCQ")
            .put(b"key0", b"BYwertw1CQ")
            .put(b"key_", b"BY1C234Q")
            .put(b"key4", b"BY1xcvbCQ")
            .put(b"key5", b"BY1gjhkjCQ")
            .put(b"key1", b"BY1CyuitQ")
            .put(b"key8", b"BY1CvbncvQ")
            .put(b"key4", b"BY1CsafQ")
            .put(b"name", b"BH1XUwqrW")
            .put(b"site", b"githuzxcvb");

        let ret = db.write(&WriteOptions::default(), batch);
        assert!(ret.is_ok());

        assert!(db.compact_range(&Default::default(), ..).is_ok());

        {
            for (k, v) in db.new_iterator(&ReadOptions::default().pin_data(true)).into_iter() {
                println!("> {:?} => {:?}", String::from_utf8_lossy(k), String::from_utf8_lossy(v));
            }
        }

        assert!(ret.is_ok());
        {
            // must pin_data
            let kvs = db
                .new_iterator(&ReadOptions::default().pin_data(true))
                .into_iter()
                .map(|(k, v)| (String::from_utf8_lossy(k), String::from_utf8_lossy(v)))
                .collect::<Vec<_>>();
            println!("got kv => {:?}", kvs);
        }

        let mut it = db.new_iterator(&ReadOptions::default().pin_data(true));

        assert_eq!(it.is_valid(), false);
        println!("it => {:?}", it);

        // FIXME: is this right?
        assert_eq!(
            it.get_property("rocksdb.iterator.is-key-pinned"),
            Ok("Iterator is not valid.".to_string())
        );

        it.seek_to_first();
        assert_eq!(it.get_property("rocksdb.iterator.is-key-pinned"), Ok("1".to_string()));

        println!(
            "got => {:?}",
            it.get_property("rocksdb.iterator.super-version-number").unwrap()
        );

        assert_eq!(it.is_valid(), true);
        println!("it => {:?}", it);
        it.next();
        println!("it => {:?}", it);
        it.seek_to_last();
        println!("it => {:?}", it);
        it.next();
        println!("it => {:?}", it);
    }

    #[test]
    fn reversed_iterator() {
        use tempdir::TempDir;
        let tmp_dir = TempDir::new_in(".", "rocks").unwrap();
        let opt = Options::default().map_db_options(|db| db.create_if_missing(true));
        let db = DB::open(opt, tmp_dir.path()).unwrap();

        let mut batch = WriteBatch::new();
        batch
            .put(b"k1", b"BYasdf1CQ")
            .put(b"k8", b"BY1C234Q")
            .put(b"k4", b"BY1dfsgCQ")
            .put(b"k2", b"BYasdf1CQ")
            .put(b"k3", b"BYasdf1CQ")
            .put(b"k6", b"BYwertw1CQ")
            .put(b"k5", b"BY1ghCQ")
            .put(b"k9", b"BY1xcvbCQ");

        let ret = db.write(&WriteOptions::default(), batch);
        assert!(ret.is_ok());

        let keys: Vec<_> = db
            .new_iterator(&ReadOptions::default().pin_data(true))
            .into_rev_iter()
            .map(|(k, _)| String::from_utf8_lossy(k).to_owned().to_string())
            .collect();
        assert_eq!(keys, vec!["k9", "k8", "k6", "k5", "k4", "k3", "k2", "k1"]);
    }
}
