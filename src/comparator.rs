//! A Comparator object provides a total order across slices that are
//! used as keys in an sstable or a database.

use std::mem;
use std::slice;
use std::os::raw::{c_char, c_int};
use std::cmp::Ordering;
use std::str;

use rocks_sys as ll;

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
    fn find_shortest_separator(&self, start: &[u8], limit: &[u8]) -> Option<&[u8]> {
        None
    }

    /// Changes `*key` to a short string `>= *key`.
    ///
    /// Simple comparator implementations may return with `*key` unchanged,
    /// i.e., an implementation of this method that does nothing is correct.
    fn find_short_successor(&self, key: &[u8]) -> Option<&[u8]> {
        None
    }
}

#[doc(hidden)]
pub mod c {
    use super::*;

    #[no_mangle]
    pub unsafe extern "C" fn rust_comparator_compare(cp: *mut (), a: *const &[u8], b: *const &[u8]) -> c_int {
        let comparator = cp as *mut &dyn Comparator;
        // FIXME: 8 byte Ordering
        mem::transmute::<_, i8>((*comparator).compare(*a, *b)) as c_int
    }


    #[no_mangle]
    pub unsafe extern "C" fn rust_comparator_equal(cp: *mut (), a: *const &[u8], b: *const &[u8]) -> c_char {
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

        let start_ptr = ll::cxx_string_data(start as *const _);
        let start_len = ll::cxx_string_size(start as *const _);

        let ret =
            (*comparator).find_shortest_separator(slice::from_raw_parts(start_ptr as *const _, start_len), *limit);
        if let Some(new_start) = ret {
            ll::cxx_string_assign(start as *mut _, new_start.as_ptr() as *const _, new_start.len())
        }
    }



    #[no_mangle]
    pub unsafe extern "C" fn rust_comparator_find_short_successor(cp: *mut (), key: *mut ()) {
        // std::string*
        let comparator = cp as *mut &dyn Comparator;

        let key_ptr = ll::cxx_string_data(key as *const _);
        let key_len = ll::cxx_string_size(key as *const _);

        let ret = (*comparator).find_short_successor(slice::from_raw_parts(key_ptr as *const _, key_len));
        if let Some(new_key) = ret {
            ll::cxx_string_assign(key as *mut _, new_key.as_ptr() as *const _, new_key.len());
        }
    }


    #[no_mangle]
    pub unsafe extern "C" fn rust_comparator_drop(op: *mut ()) {
        assert!(!op.is_null());
        let operator = op as *mut &dyn Comparator;
        Box::from_raw(operator);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::rocksdb::*;

    #[test]
    fn bitwise_comparator_normal() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(
            Options::default()
                .map_db_options(|db| db.create_if_missing(true))
                .map_cf_options(|cf| cf.bitwise_comparator_reversed(false)),
            tmp_dir,
        ).unwrap();

        assert!(db.put(&WriteOptions::default(), b"key1", b"val2").is_ok());
        assert!(db.put(&WriteOptions::default(), b"key2", b"val2").is_ok());

        let mut it = db.new_iterator(&ReadOptions::default());

        it.seek_to_first();
        assert!(it.is_valid());

        let first = it.key().to_vec();
        it.next();
        assert!(it.is_valid());

        let second = it.key().to_vec();
        assert!(first < second);
    }

    #[test]
    fn bitwise_comparator_reversed() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(
            Options::default()
                .map_db_options(|db| db.create_if_missing(true))
                .map_cf_options(|cf| cf.bitwise_comparator_reversed(true)),
            tmp_dir,
        ).unwrap();

        assert!(db.put(&WriteOptions::default(), b"key1", b"").is_ok());
        assert!(db.put(&WriteOptions::default(), b"key2", b"").is_ok());

        let mut it = db.new_iterator(&ReadOptions::default());

        it.seek_to_first();
        assert!(it.is_valid());

        let first = it.key().to_vec();
        it.next();
        assert!(it.is_valid());

        let second = it.key().to_vec();
        assert!(first > second);
    }


    pub struct MyComparator;

    impl Comparator for MyComparator {
        fn compare(&self, a: &[u8], b: &[u8]) -> Ordering {
            let sa = unsafe { str::from_utf8_unchecked(a) };
            let sb = unsafe { str::from_utf8_unchecked(b) };
            sa.to_lowercase().cmp(&sb.to_lowercase())
        }
    }

    lazy_static! {
        static ref CMP: MyComparator = {
            MyComparator
        };
    }

    #[test]
    fn custom_lowercase_comparator() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();

        let opts = Options::default()
            .map_db_options(|db| db.create_if_missing(true))
            .map_cf_options(|cf| cf.comparator(&*CMP));
        let db = DB::open(opts, tmp_dir).unwrap();

        assert!(db.put(&WriteOptions::default(), b"Key1", b"").is_ok());
        assert!(db.put(&WriteOptions::default(), b"kEY3", b"").is_ok());
        assert!(db.put(&WriteOptions::default(), b"key4", b"").is_ok());
        assert!(db.put(&WriteOptions::default(), b"kEy2", b"").is_ok());

        let ks = db.new_iterator(&ReadOptions::default().pin_data(true))
            .into_iter()
            .map(|kv| String::from_utf8_lossy(kv.0))
            .collect::<Vec<_>>();

        // println!("keys => {:?}", ks);
        assert_eq!(ks, vec!["Key1", "kEy2", "kEY3", "key4"]);
    }
}
