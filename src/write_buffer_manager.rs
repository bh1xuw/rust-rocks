
//! `WriteBufferManager` is for managing memory allocation for one or more
//! MemTables.

use rocks_sys as ll;

use to_raw::ToRaw;

/// `WriteBufferManager` is for managing memory allocation for one or more
/// MemTables.
pub struct WriteBufferManager {
    raw: *mut ll::rocks_write_buffer_manager_t,
}

impl ToRaw<ll::rocks_write_buffer_manager_t> for WriteBufferManager {
    fn raw(&self) -> *mut ll::rocks_write_buffer_manager_t {
        self.raw
    }
}

impl Drop for WriteBufferManager {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_write_buffer_manager_destroy(self.raw);
        }
    }
}

impl WriteBufferManager {
    /// _buffer_size = 0 indicates no limit. Memory won't be tracked,
    /// memory_usage() won't be valid and ShouldFlush() will always return true.
    pub fn new(buffer_size: usize) -> WriteBufferManager {
        WriteBufferManager { raw: unsafe { ll::rocks_write_buffer_manager_create(buffer_size) } }
    }

    pub fn enabled(&self) -> bool {
        unsafe { ll::rocks_write_buffer_manager_enabled(self.raw) != 0 }
    }

    // Only valid if enabled()
    pub fn memory_usage(&self) -> usize {
        unsafe { ll::rocks_write_buffer_manager_memory_usage(self.raw) }
    }

    pub fn buffer_size(&self) -> usize {
        unsafe { ll::rocks_write_buffer_manager_buffer_size(self.raw) }
    }
}

#[cfg(test)]
mod tests {
    use std::iter;
    use super::*;
    use super::super::rocksdb::*;

    #[test]
    fn write_buffer_manager_of_2db() {
        let tmp_dir1 = ::tempdir::TempDir::new_in("", "rocks").unwrap();
        let tmp_dir2 = ::tempdir::TempDir::new_in("", "rocks").unwrap();
        let manager = WriteBufferManager::new(2 << 20);

        assert_eq!(manager.memory_usage(), 0);

        let db1 = DB::open(
            Options::default().map_db_options(|db| db.create_if_missing(true).write_buffer_manager(&manager)),
            &tmp_dir1,
        ).unwrap();

        let mem1 = manager.memory_usage();

        let db2 = DB::open(
            Options::default().map_db_options(|db| db.create_if_missing(true).write_buffer_manager(&manager)),
            &tmp_dir2,
        ).unwrap();


        assert_eq!(manager.enabled(), true);
        let mem2 = manager.memory_usage();
        assert!(mem2 > mem1);

        for i in 0..100 {
            let key = format!("k{}", i);
            let val = format!("v{}", i * i);
            let value: String = iter::repeat(val).take(i * i).collect::<Vec<_>>().concat();

            db1.put(WriteOptions::default_instance(), key.as_bytes(), value.as_bytes())
                .unwrap();
        }

        let mem3 = manager.memory_usage();
        assert!(mem3 > mem2);

        for i in 0..100 {
            let key = format!("k{}", i);
            let val = format!("v{}", i * i);
            let value: String = iter::repeat(val).take(i * i).collect::<Vec<_>>().concat();

            db2.put(WriteOptions::default_instance(), key.as_bytes(), value.as_bytes())
                .unwrap();
        }

        let mem4 = manager.memory_usage();
        assert!(mem4 > mem3);

        assert!(db2.flush(&Default::default()).is_ok());
        let mem5 = manager.memory_usage();
        assert!(mem5 < mem4);
        drop(db1);
        drop(db2);
        assert_eq!(manager.memory_usage(), 0);
    }
}
