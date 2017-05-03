
use rocks_sys as ll;

use status::Status;
use options::ColumnFamilyOptions;
use comparator::Comparator;
use options::Options;
use table_properties::TableProperties;

const DEFAULT_COLUMN_FAMILY_NAME: &'static str = "default";


pub struct ColumnFamilyDescriptor {
    name: String,
    options: ColumnFamilyOptions,
}

impl ColumnFamilyDescriptor {
    pub fn new(name: &str, options: &ColumnFamilyOptions) -> ColumnFamilyDescriptor {
        unimplemented!()
    }
}

impl Default for ColumnFamilyDescriptor {
    fn default() -> Self {
        ColumnFamilyDescriptor::new(DEFAULT_COLUMN_FAMILY_NAME, &ColumnFamilyOptions::default())
    }
}

pub struct ColumnFamilyHandle {
}

impl ColumnFamilyHandle {
    /// Returns the name of the column family associated with the current handle.
    pub fn get_name(&self) -> &'static str {
        unimplemented!()
    }

    /// Returns the ID of the column family associated with the current handle.
    pub fn get_id(&self) -> u32 {
        unimplemented!()
    }

    /// Fills "*desc" with the up-to-date descriptor of the column family
    /// associated with this handle. Since it fills "*desc" with the up-to-date
    /// information, this call might internally lock and release DB mutex to
    /// access the up-to-date CF options.  In addition, all the pointer-typed
    /// options cannot be referenced any longer than the original options exist.
    ///
    /// Note that this function is not supported in RocksDBLite.
    pub fn get_descriptor(&self, desc: &mut ColumnFamilyDescriptor) -> Status {
        Status::new()
    }

    /// Returns the comparator of the column family associated with the
    /// current handle.
    pub fn get_comparator(&self) -> Comparator {
        unimplemented!()
    }
}



/// A range of keys
pub struct Range {
    /// Included in the range
    start: Vec<u8>,
    /// Not included in the range
    limit: Vec<u8>,
}


impl Range {
    pub fn new(s: &[u8], l: &[u8]) -> Range {
        Range {
            start: s.to_owned(),
            limit: l.to_owned(),
        }
    }
}

/// A DB is a persistent ordered map from keys to values.
/// A DB is safe for concurrent access from multiple threads without
/// any external synchronization.
pub struct DB {
    raw: *mut ll::db::DB,
}

impl DB {
    /// Open the database with the specified "name".
    /// Stores a pointer to a heap-allocated database in *dbptr and returns
    /// OK on success.
    /// Stores nullptr in *dbptr and returns a non-OK status on error.
    /// Caller should delete *dbptr when it is no longer needed.
    pub fn open(options: &Options, name: &str) -> Result<DB, Status> {
        unimplemented!()
    }

    // Open the database for read only. All DB interfaces
    // that modify data, like put/delete, will return error.
    // If the db is opened in read only mode, then no compactions
    // will happen.
    //
    // Not supported in ROCKSDB_LITE, in which case the function will
    // return Status::NotSupported.
    pub fn open_for_readonly(options: &Options, name: &str) -> Result<DB, Status> {
        unimplemented!()
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
