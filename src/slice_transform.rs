//! Class for specifying user-defined functions which perform a
//! transformation on a slice.
//!
//! It is not required that every slice
//! belong to the domain and/or range of a function.  Subclasses should
//! define InDomain and InRange to determine which slices are in either
//! of these sets respectively.

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

#[cfg(test)]
mod tests {
    use super::super::rocksdb::*;
    use super::*;

    pub struct MySliceTransform;

    impl SliceTransform for MySliceTransform {
        // assume key in format: XX-prefix-whatever
        fn transform<'a>(&self, key: &'a [u8]) -> &'a [u8] {
            assert!(key.len() > 10);
            &key[3..9]
        }
    }

    // FIXME: useless?
    #[test]
    fn customized_prefix_extractor() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(
            Options::default()
                .map_db_options(|db| db.create_if_missing(true))
                .map_cf_options(|cf| {
                    cf.prefix_extractor(Box::new(MySliceTransform))
                        .memtable_prefix_bloom_size_ratio(0.1) // enable prefix bloom filter
                }),
            &tmp_dir,
        )
        .unwrap();

        // NOTE: all these has same prefixes
        // if use another prefix, iterator may be wrong.
        // since it will find a non-included key and skip following.
        // FOR TEST ONLY, this kind of prefix extractor is joking!
        assert!(db.put(&WriteOptions::default(), b"AA-abcdef-003", b"23333").is_ok());
        assert!(db.put(&WriteOptions::default(), b"AA-abcdef-001", b"23333").is_ok());
        assert!(db.put(&WriteOptions::default(), b"AA-abcdef-002", b"23333").is_ok());
        assert!(db.put(&WriteOptions::default(), b"BB-abcdef-005", b"23333").is_ok());
        assert!(db.put(&WriteOptions::default(), b"AA-abcdef-002", b"23333").is_ok());
        assert!(db.put(&WriteOptions::default(), b"CC-abcdef-001", b"23333").is_ok());

        let mut it = db.new_iterator(&ReadOptions::default().pin_data(true).prefix_same_as_start(true));
        it.seek(b"---abcdef--");

        assert!(it.is_valid());

        let mut keys = vec![];
        while it.is_valid() {
            keys.push(String::from_utf8_lossy(it.key()).to_owned().to_string());
            it.next();
        }

        assert!(keys.contains(&"CC-abcdef-001".to_string()));
        assert!(keys.contains(&"BB-abcdef-005".to_string()));
        assert!(keys.contains(&"AA-abcdef-002".to_string()));
    }

    #[test]
    fn prefix_extractor_capped() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(
            Options::default()
                .map_db_options(|db| db.create_if_missing(true))
                .map_cf_options(|cf| {
                    cf.prefix_extractor_capped(3) // first 3 chars
                        .memtable_prefix_bloom_size_ratio(0.1) // enable prefix bloom filter
                }),
            &tmp_dir,
        )
        .unwrap();

        assert!(db.put(&WriteOptions::default(), b"abc-003", b"23333").is_ok());
        assert!(db.put(&WriteOptions::default(), b"abc-001", b"23333").is_ok());
        assert!(db.put(&WriteOptions::default(), b"abc-002", b"23333").is_ok());
        assert!(db.put(&WriteOptions::default(), b"abc-005", b"23333").is_ok());
        assert!(db.put(&WriteOptions::default(), b"abc-002", b"23333").is_ok());
        assert!(db.put(&WriteOptions::default(), b"abc-006", b"23333").is_ok());
        assert!(db.put(&WriteOptions::default(), b"def-000", b"23333").is_ok());

        let mut it = db.new_iterator(&ReadOptions::default().pin_data(true).prefix_same_as_start(true));
        it.seek(b"abc-");

        assert!(it.is_valid());

        let mut keys = vec![];
        while it.is_valid() {
            keys.push(String::from_utf8_lossy(it.key()).to_owned().to_string());
            it.next();
        }

        assert!(keys.contains(&"abc-001".to_string()));
        assert!(keys.contains(&"abc-005".to_string()));
        assert!(keys.contains(&"abc-002".to_string()));
        assert!(!keys.contains(&"def-000".to_string()));
    }
}
