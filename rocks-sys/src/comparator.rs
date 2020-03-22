use std::cmp::Ordering;
use std::mem;
use std::os::raw::{c_char, c_int};
use std::slice;
use std::str;

/// A `Comparator` object provides a total order across slices that are
/// used as keys in an sstable or a database. A `Comparator` implementation
/// must be thread-safe since rocksdb may invoke its methods concurrently
/// from multiple threads.
pub trait Comparator {
    /// Three-way comparison.  Returns value:
    ///
    /// - `< 0 iff "a" < "b"`,
    /// - `== 0 iff "a" == "b"`,
    /// - `> 0 iff "a" > "b"`
    fn compare(&self, a: &[u8], b: &[u8]) -> Ordering;

    /// Compares two slices for equality. The following invariant should always
    /// hold (and is the default implementation):
    ///
    /// > `Equal(a, b) iff Compare(a, b) == 0`
    ///
    /// Overwrite only if equality comparisons can be done more efficiently than
    /// three-way comparisons.
    fn equal(&self, a: &[u8], b: &[u8]) -> bool {
        self.compare(a, b) == Ordering::Equal
    }
    /// The name of the comparator.  Used to check for comparator
    /// mismatches (i.e., a DB created with one comparator is
    /// accessed using a different comparator.
    ///
    /// The client of this package should switch to a new name whenever
    /// the comparator implementation changes in a way that will cause
    /// the relative ordering of any two keys to change.
    ///
    /// Names starting with "rocksdb." are reserved and should not be used
    /// by any clients of this package.
    // FIXME: \0 ended
    fn name(&self) -> &str {
        "rust-rocks.Comparator\0"
    }

    // Advanced functions: these are used to reduce the space requirements
    // for internal data structures like index blocks.

    /// If `*start < limit`, changes `*start` to a short string in `[start,limit)`.
    /// Simple comparator implementations may return with `*start` unchanged,
    /// i.e., an implementation of this method that does nothing is correct.
    fn find_shortest_separator(&self, _start: &[u8], _limit: &[u8]) -> Option<&[u8]> {
        None
    }

    /// Changes `*key` to a short string `>= *key`.
    ///
    /// Simple comparator implementations may return with `*key` unchanged,
    /// i.e., an implementation of this method that does nothing is correct.
    fn find_short_successor(&self, _key: &[u8]) -> Option<&[u8]> {
        None
    }
}

#[doc(hidden)]
pub mod rust_export {
    use super::*;

    #[no_mangle]
    pub unsafe extern "C" fn rust_comparator_compare(
        cp: *mut (),
        a: *const &[u8],
        b: *const &[u8],
    ) -> c_int {
        let comparator = cp as *mut &dyn Comparator;
        // FIXME: 8 byte Ordering
        mem::transmute::<_, i8>((*comparator).compare(*a, *b)) as c_int
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_comparator_equal(
        cp: *mut (),
        a: *const &[u8],
        b: *const &[u8],
    ) -> c_char {
        let comparator = cp as *mut &dyn Comparator;
        ((*comparator).equal(*a, *b)) as c_char
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_comparator_name(cp: *mut ()) -> *const c_char {
        let comparator = cp as *mut &dyn Comparator;
        (*comparator).name().as_ptr() as *const _
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_comparator_find_shortest_separator(
        cp: *mut (),
        start: *mut (), // std::string*
        limit: *const &[u8],
    ) {
        // Slice&
        let comparator = cp as *mut &dyn Comparator;

        let start_ptr = crate::cxx_string_data(start as *const _);
        let start_len = crate::cxx_string_size(start as *const _);

        let ret = (*comparator).find_shortest_separator(
            slice::from_raw_parts(start_ptr as *const _, start_len),
            *limit,
        );
        if let Some(new_start) = ret {
            crate::cxx_string_assign(
                start as *mut _,
                new_start.as_ptr() as *const _,
                new_start.len(),
            )
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_comparator_find_short_successor(cp: *mut (), key: *mut ()) {
        // std::string*
        let comparator = cp as *mut &dyn Comparator;

        let key_ptr = crate::cxx_string_data(key as *const _);
        let key_len = crate::cxx_string_size(key as *const _);

        let ret =
            (*comparator).find_short_successor(slice::from_raw_parts(key_ptr as *const _, key_len));
        if let Some(new_key) = ret {
            crate::cxx_string_assign(key as *mut _, new_key.as_ptr() as *const _, new_key.len());
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_comparator_drop(op: *mut ()) {
        assert!(!op.is_null());
        let operator = op as *mut &dyn Comparator;
        Box::from_raw(operator);
    }
}
