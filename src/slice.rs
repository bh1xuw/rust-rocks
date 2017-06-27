//! Slice data structure for interacting with rocksdb keys and values.

use std::fmt;
use std::slice;
use std::ops;
use std::str;

use rocks_sys as ll;

use to_raw::ToRaw;

pub struct CVec<T> {
    data: *mut T,
    len: usize,
}

impl<T> CVec<T> {
    pub unsafe fn from_raw_parts(p: *mut T, len: usize) -> CVec<T> {
        CVec {
            data: p,
            len: len,
        }
    }
}

impl CVec<u8> {
    pub fn to_str(&self) -> Result<&str, str::Utf8Error> {
        str::from_utf8(self)
    }
}

impl<T: fmt::Debug> fmt::Debug for CVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe { slice::from_raw_parts(self.data, self.len).fmt(f) }
    }
}

impl<T> ops::Deref for CVec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.data, self.len) }
    }
}

impl<T> AsRef<[T]> for CVec<T> {
    fn as_ref(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.data, self.len) }
    }
}

impl<T> Drop for CVec<T> {
    fn drop(&mut self) {
        unsafe {
            ll::free(self.data as _);
        }
    }
}

impl<'a, T: PartialEq> PartialEq<&'a [T]> for CVec<T> {
    fn eq(&self, rhs: &&[T]) -> bool {
        &self.as_ref() == rhs
    }
}

impl<'a, 'b, T: PartialEq> PartialEq<&'b [T]> for &'a CVec<T> {
    fn eq(&self, rhs: &&[T]) -> bool {
        &self.as_ref() == rhs
    }
}


/// A Slice that can be pinned with some cleanup tasks, which will be run upon
/// `::Reset()` or object destruction, whichever is invoked first. This can be used
/// to avoid memcpy by having the `PinnsableSlice` object referring to the data
/// that is locked in the memory and release them after the data is consuned.
pub struct PinnableSlice {
    raw: *mut ll::rocks_pinnable_slice_t,
}

impl ToRaw<ll::rocks_pinnable_slice_t> for PinnableSlice {
    fn raw(&self) -> *mut ll::rocks_pinnable_slice_t {
        self.raw
    }
}

impl Drop for PinnableSlice {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_pinnable_slice_destroy(self.raw);
        }
    }
}

impl PinnableSlice {
    pub fn new() -> PinnableSlice {
        PinnableSlice { raw: unsafe { ll::rocks_pinnable_slice_create() } }
    }

    #[inline]
    pub fn data(&self) -> *const u8 {
        unsafe { ll::rocks_pinnable_slice_data(self.raw) as *const u8 }
    }

    #[inline]
    pub fn size(&self) -> usize {
        unsafe { ll::rocks_pinnable_slice_size(self.raw) as usize }
    }
}


impl fmt::Debug for PinnableSlice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = unsafe { slice::from_raw_parts(self.data(), self.len()) };
        write!(f, "{:?}", String::from_utf8_lossy(s))
    }
}

impl Default for PinnableSlice {
    fn default() -> Self {
        PinnableSlice::new()
    }
}

impl ops::Deref for PinnableSlice {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data(), self.size()) }
    }
}

impl AsRef<[u8]> for PinnableSlice {
    fn as_ref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data(), self.len()) }
    }
}

impl<'a> PartialEq<&'a [u8]> for PinnableSlice {
    fn eq(&self, rhs: &&[u8]) -> bool {
        &self.as_ref() == rhs
    }
}

impl<'a, 'b> PartialEq<&'b [u8]> for &'a PinnableSlice {
    fn eq(&self, rhs: &&[u8]) -> bool {
        &self.as_ref() == rhs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinnable_slice() {
        let s = PinnableSlice::new();
        assert_eq!(s, b"");
        assert_eq!(&format!("{:?}", s), "\"\"");
    }
}
