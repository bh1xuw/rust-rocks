
use rocks_sys as ll;

use options::Options;


pub struct DbDumpTool {
    raw: *mut ll::rocks_dump_options_t,
}

impl Drop for DbDumpTool {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_dump_options_destroy(self.raw)
        }
    }
}

impl DbDumpTool {
    pub fn new(db_path: &str, dump_location: &str) -> DbDumpTool {
        unsafe {
            let raw = ll::rocks_dump_options_create();
            ll::rocks_dump_options_set_db_path(raw,
                                               db_path.as_bytes().as_ptr() as *const _,
                                               db_path.as_bytes().len());
            ll::rocks_dump_options_set_dump_location(raw,
                                                     dump_location.as_bytes().as_ptr() as *const _,
                                                     dump_location.as_bytes().len());
            DbDumpTool {
                raw: raw
            }
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
        unsafe {
            ll::rocks_db_dump_tool_run(self.raw, options.raw()) != 0
        }
    }
}



pub struct DbUndumpTool {
    raw: *mut ll::rocks_undump_options_t,
}

impl Drop for DbUndumpTool {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_undump_options_destroy(self.raw)
        }
    }
}

impl DbUndumpTool {
    pub fn new(db_path: &str, dump_location: &str) -> DbUndumpTool {
        unsafe {
            let raw = ll::rocks_undump_options_create();
            ll::rocks_undump_options_set_db_path(raw,
                                               db_path.as_bytes().as_ptr() as *const _,
                                               db_path.as_bytes().len());
            ll::rocks_undump_options_set_dump_location(raw,
                                                       dump_location.as_bytes().as_ptr() as *const _,
                                                       dump_location.as_bytes().len());
            DbUndumpTool {
                raw: raw
            }
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
        unsafe {
            ll::rocks_db_undump_tool_run(self.raw, options.raw()) != 0
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::rocksdb::*;

    // TODO: create a test db
    #[test]
    fn db_dump_and_undump() {
        let dump_ok = DbDumpTool::new("./data", "./output.dump")
            .run(&Options::default());

        assert!(dump_ok);

        let undump_ok = DbUndumpTool::new("./data2", "./output.dump")
            .run(&Options::default());

        assert!(undump_ok);
    }
}
