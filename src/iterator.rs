/// An iterator yields a sequence of key/value pairs from a source.
/// The following class defines the interface.  Multiple implementations
/// are provided by this library.  In particular, iterators are provided
/// to access the contents of a Table or a DB.
///
/// Multiple threads can invoke const methods on an Iterator without
/// external synchronization, but if any of the threads may call a
/// non-const method, all threads accessing the same Iterator must use
/// external synchronization.

use std::mem;
use std::slice;
use std::fmt;
use std::iter;
use std::marker::PhantomData;

use rocks_sys as ll;

use status::Status;

pub struct Iterator {
    raw: *mut ll::rocks_iterator_t,
}

impl fmt::Debug for Iterator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "Iterator("));
        if self.is_valid() {
            write!(f,
                   "key={:?} value={:?})",
                   String::from_utf8_lossy(self.key()),
                   String::from_utf8_lossy(self.value()))
        } else {
            write!(f, "INVALID)")
        }
    }
}


impl Drop for Iterator {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_iter_destroy(self.raw);
        }
    }
}

impl Iterator {
    pub unsafe fn from_ll(raw: *mut ll::rocks_iterator_t) -> Self {
        Iterator { raw: raw }
    }

    /// An iterator is either positioned at a key/value pair, or
    /// not valid.  This method returns true iff the iterator is valid.
    pub fn is_valid(&self) -> bool {
        unsafe { ll::rocks_iter_valid(self.raw) != 0 }
    }

    /// Position at the first key in the source.  The iterator is Valid()
    /// after this call iff the source is not empty.
    pub fn seek_to_first(&mut self) {
        unsafe {
            ll::rocks_iter_seek_to_first(self.raw);
        }
    }

    /// Position at the last key in the source.  The iterator is
    /// Valid() after this call iff the source is not empty.
    pub fn seek_to_last(&mut self) {
        unsafe {
            ll::rocks_iter_seek_to_last(self.raw);
        }
    }

    /// Position at the first key in the source that at or past target
    /// The iterator is Valid() after this call iff the source contains
    /// an entry that comes at or past target.
    pub fn seek(&mut self, target: &[u8]) {
        unsafe {
            ll::rocks_iter_seek(self.raw, target.as_ptr() as _, target.len());
        }
    }

    /// Position at the last key in the source that at or before target
    /// The iterator is Valid() after this call iff the source contains
    /// an entry that comes at or before target.
    pub fn seek_for_prev(&mut self, target: &[u8]) {
        unsafe {
            ll::rocks_iter_seek_for_prev(self.raw, target.as_ptr() as _, target.len());
        }
    }

    /// Moves to the next entry in the source.  After this call, Valid() is
    /// true iff the iterator was not positioned at the last entry in the source.
    /// REQUIRES: Valid()
    pub fn next(&mut self) {
        unsafe {
            ll::rocks_iter_next(self.raw);
        }
    }

    /// Moves to the previous entry in the source.  After this call, Valid() is
    /// true iff the iterator was not positioned at the first entry in source.
    /// REQUIRES: Valid()
    pub fn prev(&mut self) {
        unsafe {
            ll::rocks_iter_prev(self.raw);
        }
    }

    /// Return the key for the current entry.  The underlying storage for
    /// the returned slice is valid only until the next modification of
    /// the iterator.
    /// REQUIRES: Valid()
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
    /// REQUIRES: !AtEnd() && !AtStart()
    pub fn value(&self) -> &[u8] {
        unsafe {
            let mut len = 0;
            let ptr = ll::rocks_iter_value(self.raw, &mut len);
            slice::from_raw_parts(ptr as _, len)
        }
    }

    /// If an error has occurred, return it.  Else return an ok status.
    /// If non-blocking IO is requested and this operation cannot be
    /// satisfied without doing some IO, then this returns Status::Incomplete().
    pub fn get_status(&self) -> Status {
        unsafe {
            let mut status = mem::zeroed();
            ll::rocks_iter_get_status(self.raw, &mut status);
            Status::from_ll(&status)
        }
    }

    /// Property "rocksdb.iterator.is-key-pinned":
    ///   If returning "1", this means that the Slice returned by key() is valid
    ///   as long as the iterator is not deleted.
    ///   It is guaranteed to always return "1" if
    ///      - Iterator created with ReadOptions::pin_data = true
    ///      - DB tables were created with
    ///        BlockBasedTableOptions::use_delta_encoding = false.
    /// Property "rocksdb.iterator.super-version-number":
    ///   LSM version used by the iterator. The same format as DB Property
    ///   kCurrentSuperVersionNumber. See its comment for more information.
    pub fn get_property(&self, prop_name: &str) -> String {
        unimplemented!()
    }

    // FIXME: leaks?
    pub fn iter<'a>(mut self) -> Iter<'a> {
        if !self.is_valid() {
            self.seek_to_first();
        }
        Iter {
            inner: self,
            _marker: PhantomData,
        }
    }
}


// Wraps into a rust-style Iterator
pub struct Iter<'a> {
    inner: Iterator,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Iter<'a> {
    pub fn into_inner(self) -> Iterator {
        self.inner
    }
}

impl<'a> iter::Iterator for Iter<'a> {
    type Item = (&'a [u8], &'a [u8]);

    // FIXME: is it dangerous if data is un-pinned?
    fn next(&mut self) -> Option<Self::Item> {
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
