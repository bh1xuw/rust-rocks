//! Class for specifying user-defined functions which perform a
//! transformation on a slice.
//!
//! It is not required that every slice
//! belong to the domain and/or range of a function.  Subclasses should
//! define InDomain and InRange to determine which slices are in either
//! of these sets respectively.

#[doc(inline)]
pub use rocks_sys::slice_transform::SliceTransform;

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
    fn prefix_extractor_customized() {
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
        assert!(db
            .put(&WriteOptions::default(), b"AA-abcdef-003", b"23333")
            .is_ok());
        assert!(db
            .put(&WriteOptions::default(), b"AA-abcdef-001", b"23333")
            .is_ok());
        assert!(db
            .put(&WriteOptions::default(), b"AA-abcdef-002", b"23333")
            .is_ok());
        assert!(db
            .put(&WriteOptions::default(), b"BB-abcdef-005", b"23333")
            .is_ok());
        assert!(db
            .put(&WriteOptions::default(), b"AA-abcdef-002", b"23333")
            .is_ok());
        assert!(db
            .put(&WriteOptions::default(), b"CC-abcdef-001", b"23333")
            .is_ok());

        let mut it = db.new_iterator(&ReadOptions::default().prefix_same_as_start(true));
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

        assert!(db
            .put(&WriteOptions::default(), b"abc-003", b"23333")
            .is_ok());
        assert!(db
            .put(&WriteOptions::default(), b"abc-001", b"23333")
            .is_ok());
        assert!(db
            .put(&WriteOptions::default(), b"abc-002", b"23333")
            .is_ok());
        assert!(db
            .put(&WriteOptions::default(), b"abc-005", b"23333")
            .is_ok());
        assert!(db
            .put(&WriteOptions::default(), b"abc-002", b"23333")
            .is_ok());
        assert!(db
            .put(&WriteOptions::default(), b"abc-006", b"23333")
            .is_ok());
        assert!(db
            .put(&WriteOptions::default(), b"def-000", b"23333")
            .is_ok());

        let mut it = db.new_iterator(&ReadOptions::default().prefix_same_as_start(true));
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
