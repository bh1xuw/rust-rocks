//! A Comparator object provides a total order across slices that are
//! used as keys in an sstable or a database.

#[doc(inline)]
pub use rocks_sys::comparator::Comparator;

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;
    use std::str;

    use super::super::rocksdb::*;
    use super::*;

    #[test]
    fn bitwise_comparator_normal() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(
            Options::default()
                .map_db_options(|db| db.create_if_missing(true))
                .map_cf_options(|cf| cf.bitwise_comparator_reversed(false)),
            tmp_dir,
        )
        .unwrap();

        assert!(db.put(&WriteOptions::default(), b"key1", b"val2").is_ok());
        assert!(db.put(&WriteOptions::default(), b"key2", b"val2").is_ok());

        let mut it = db.new_iterator(&ReadOptions::default().pin_data(true));

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
        )
        .unwrap();

        assert!(db.put(&WriteOptions::default(), b"key1", b"").is_ok());
        assert!(db.put(&WriteOptions::default(), b"key2", b"").is_ok());

        let mut it = db.new_iterator(&ReadOptions::default().pin_data(true));

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
        fn compare(&self, a: &[u8], b: &[u8]) -> ::std::cmp::Ordering {
            let sa = unsafe { str::from_utf8_unchecked(a) };
            let sb = unsafe { str::from_utf8_unchecked(b) };
            sa.to_lowercase().cmp(&sb.to_lowercase())
        }
    }

    lazy_static! {
        static ref CMP: MyComparator = { MyComparator };
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

        let ks = db
            .new_iterator(&ReadOptions::default().pin_data(true))
            .into_iter()
            .map(|kv| String::from_utf8_lossy(kv.0))
            .collect::<Vec<_>>();

        // println!("keys => {:?}", ks);
        assert_eq!(ks, vec!["Key1", "kEy2", "kEY3", "key4"]);
    }
}
