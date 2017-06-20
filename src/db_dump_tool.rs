//! Dump and un-dump tools for rocksdb

use std::path::Path;

use rocks_sys as ll;

use options::Options;
use to_raw::ToRaw;

/// Dumps db to a ROCKDUMP file
pub struct DbDumpTool {
    raw: *mut ll::rocks_dump_options_t,
}

impl Drop for DbDumpTool {
    fn drop(&mut self) {
        unsafe { ll::rocks_dump_options_destroy(self.raw) }
    }
}

impl DbDumpTool {
    pub fn new<P: AsRef<Path>, Q: AsRef<Path>>(db_path: P, dump_location: Q) -> DbDumpTool {
        unsafe {
            let raw = ll::rocks_dump_options_create();
            let db_path = db_path.as_ref().to_str().expect("valid path");
            let dump_location = dump_location.as_ref().to_str().expect("valid path");
            ll::rocks_dump_options_set_db_path(raw, db_path.as_bytes().as_ptr() as *const _, db_path.as_bytes().len());
            ll::rocks_dump_options_set_dump_location(
                raw,
                dump_location.as_bytes().as_ptr() as *const _,
                dump_location.as_bytes().len(),
            );
            DbDumpTool { raw: raw }
        }
    }

    /// Dont include db information header in the dump
    ///
    /// DEFAULT: false
    pub fn anonymous(self, val: bool) -> Self {
        unsafe {
            ll::rocks_dump_options_set_anonymous(self.raw, val as u8);
        }
        self
    }

    pub fn run(self, options: &Options) -> bool {
        unsafe { ll::rocks_db_dump_tool_run(self.raw, options.raw()) != 0 }
    }
}

/// Undumps(load) db from a ROCKDUMP file
pub struct DbUndumpTool {
    raw: *mut ll::rocks_undump_options_t,
}

impl Drop for DbUndumpTool {
    fn drop(&mut self) {
        unsafe { ll::rocks_undump_options_destroy(self.raw) }
    }
}

impl DbUndumpTool {
    pub fn new<P: AsRef<Path>, Q: AsRef<Path>>(db_path: P, dump_location: Q) -> DbUndumpTool {
        unsafe {
            let raw = ll::rocks_undump_options_create();
            let db_path = db_path.as_ref().to_str().expect("valid path");
            let dump_location = dump_location.as_ref().to_str().expect("valid path");
            ll::rocks_undump_options_set_db_path(
                raw,
                db_path.as_bytes().as_ptr() as *const _,
                db_path.as_bytes().len(),
            );
            ll::rocks_undump_options_set_dump_location(
                raw,
                dump_location.as_bytes().as_ptr() as *const _,
                dump_location.as_bytes().len(),
            );
            DbUndumpTool { raw: raw }
        }
    }

    /// Compact the db after loading the dumped file
    ///
    /// DEFAULT: false
    pub fn compact_db(self, val: bool) -> Self {
        unsafe {
            ll::rocks_undump_options_set_compact_db(self.raw, val as u8);
        }
        self
    }

    pub fn run(self, options: &Options) -> bool {
        unsafe { ll::rocks_db_undump_tool_run(self.raw, options.raw()) != 0 }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::rocksdb::*;

    #[test]
    fn db_dump_and_undump() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();

        {
            let opt = Options::default().map_db_options(|db| db.create_if_missing(true));
            let db = DB::open(opt, &tmp_dir).unwrap();
            let mut batch = WriteBatch::new();
            batch
                .put(b"key1", b"BYasdf1CQ")
                .put(b"key2", b"BYasdf1CQ")
                .put(b"key3", b"BYasdf1CQ")
                .put(b"key4", b"BY1dfsgCQ")
                .put(b"key5", b"BY1ghCQ")
                .put(b"key0", b"BYwertw1CQ")
                .put(b"key_", b"BY1CQ")
                .put(b"key4", b"BY1xcvbCQ")
                .put(b"key5", b"BY1gjhkjCQ")
                .put(b"key1", b"BY1CyuitQ")
                .put(b"key8", b"BY1CvbncvQ")
                .put(b"key4", b"BY1CsafQ")
                .put(b"name", b"BH1XUwqrW")
                .put(b"site", b"githuzxcvb");
            assert!(db.write(WriteOptions::default(), batch).is_ok());
        }

        let dumps_dir = ::tempdir::TempDir::new_in(".", "dumps").unwrap();

        let tmp_dir2 = ::tempdir::TempDir::new_in(".", "rocks").unwrap();

        let dump_ok = DbDumpTool::new(&tmp_dir, dumps_dir.path().join("mydump")).run(&Options::default());
        assert!(dump_ok);

        let undump_ok = DbUndumpTool::new(&tmp_dir2, dumps_dir.path().join("mydump")).run(&Options::default());
        assert!(undump_ok);

        {
            let opt = Options::default();
            let db = DB::open_for_readonly(&opt, &tmp_dir2, false).unwrap();

            assert_eq!(db.get(&Default::default(), b"key_").as_ref().unwrap(), b"BY1CQ".as_ref());
        }
    }
}
