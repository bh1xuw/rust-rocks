/// A `SliceTranform` is a generic pluggable way of transforming one string
/// to another. Its primary use-case is in configuring rocksdb
/// to store prefix blooms by setting prefix_extractor in
/// ColumnFamilyOptions.
pub trait SliceTransform {
    /// Extract a prefix from a specified key. This method is called when
    /// a key is inserted into the db, and the returned slice is used to
    /// create a bloom filter.
    fn transform<'a>(&self, key: &'a [u8]) -> &'a [u8];

    /// Determine whether the specified key is compatible with the logic
    /// specified in the Transform method. This method is invoked for every
    /// key that is inserted into the db. If this method returns true,
    /// then Transform is called to translate the key to its prefix and
    /// that returned prefix is inserted into the bloom filter. If this
    /// method returns false, then the call to Transform is skipped and
    /// no prefix is inserted into the bloom filters.
    ///
    /// For example, if the Transform method operates on a fixed length
    /// prefix of size 4, then an invocation to InDomain("abc") returns
    /// false because the specified key length(3) is shorter than the
    /// prefix size of 4.
    ///
    /// Wiki documentation here:
    /// https://github.com/facebook/rocksdb/wiki/Prefix-Seek-API-Changes
    fn in_domain(&self, _key: &[u8]) -> bool {
        true // default: use transform
    }

    /// Return the name of this transformation.
    fn name(&self) -> &str {
        "RustSliceTransform\0"
    }
}

// rust -> c part
#[doc(hidden)]
pub mod c {
    use std::os::raw::c_char;

    use super::SliceTransform;

    #[no_mangle]
    pub unsafe extern "C" fn rust_slice_transform_call(
        t: *mut (),
        key: &&[u8], // *Slice
        ret_value: *mut *const c_char,
        ret_len: *mut usize,
    ) {
        let trans = t as *mut Box<dyn SliceTransform>;
        let ret = (*trans).transform(key);
        *ret_value = ret.as_ptr() as *const _;
        *ret_len = ret.len();
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_slice_transform_name(t: *mut ()) -> *const c_char {
        let trans = t as *mut Box<dyn SliceTransform>;
        (*trans).name().as_ptr() as *const _
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_slice_transform_in_domain(t: *mut (), key: &&[u8]) -> c_char {
        let trans = t as *mut Box<dyn SliceTransform>;
        (*trans).in_domain(key) as c_char
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_slice_transform_drop(t: *mut ()) {
        let trans = t as *mut Box<dyn SliceTransform>;
        Box::from_raw(trans);
    }
}
