//! A DB is a persistent ordered map from keys to values.

use std::collections::hash_map::HashMap;
use std::ffi::{CStr, CString};
use std::fmt;
use std::iter::IntoIterator;
use std::mem;
use std::ops;
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;
use std::ptr;
use std::slice;
use std::str;
use std::sync::Arc;
use std::time::Duration;

use rocks_sys as ll;

use crate::debug::KeyVersionVec;
use crate::iterator::Iterator;
use crate::metadata::{ColumnFamilyMetaData, LevelMetaData, LiveFileMetaData, SstFileMetaData};
use crate::options::{
    ColumnFamilyOptions, CompactRangeOptions, CompactionOptions, DBOptions, FlushOptions, IngestExternalFileOptions,
    Options, ReadOptions, WriteOptions,
};
use crate::slice::PinnableSlice;
use crate::snapshot::Snapshot;
use crate::table_properties::TablePropertiesCollection;
use crate::to_raw::{FromRaw, ToRaw};
use crate::transaction_log::{LogFile, TransactionLogIterator};
use crate::types::SequenceNumber;
use crate::utilities::path_to_bytes;
use crate::write_batch::WriteBatch;
use crate::{Error, Result};

pub const DEFAULT_COLUMN_FAMILY_NAME: &'static str = "default";

/// Descriptor of a column family, name and the options
#[derive(Debug)]
pub struct ColumnFamilyDescriptor {
    name: CString,
    options: ColumnFamilyOptions,
}

impl ColumnFamilyDescriptor {
    fn with_name<T: AsRef<str>>(name: T) -> ColumnFamilyDescriptor {
        ColumnFamilyDescriptor {
            name: CString::new(name.as_ref()).expect("need a valid column family name"),
            options: ColumnFamilyOptions::default(),
        }
    }

    fn name_as_ptr(&self) -> *const c_char {
        self.name.as_ptr()
    }

    pub fn new<T: AsRef<str>>(name: T, options: ColumnFamilyOptions) -> ColumnFamilyDescriptor {
        ColumnFamilyDescriptor {
            name: CString::new(name.as_ref()).expect("need a valid column family name"),
            options,
        }
    }

    pub fn name(&self) -> &str {
        self.name.to_str().expect("non utf8 cf name")
    }

    pub fn options(&self) -> &ColumnFamilyOptions {
        &self.options
    }

    /// Configure ColumnFamilyOptions using builder style.
    pub fn map_cf_options<F: FnOnce(ColumnFamilyOptions) -> ColumnFamilyOptions>(self, f: F) -> Self {
        let ColumnFamilyDescriptor { name, options } = self;
        let new_options = f(options);
        ColumnFamilyDescriptor {
            name,
            options: new_options,
        }
    }
}

// FIXME: default column family uses default ColumnFamilyOptions
impl Default for ColumnFamilyDescriptor {
    fn default() -> Self {
        ColumnFamilyDescriptor::new(DEFAULT_COLUMN_FAMILY_NAME, ColumnFamilyOptions::default())
    }
}

impl<T: AsRef<str>> From<T> for ColumnFamilyDescriptor {
    fn from(name: T) -> Self {
        ColumnFamilyDescriptor::with_name(name)
    }
}

/// Handle for a opened column family
pub struct ColumnFamilyHandle {
    raw: *mut ll::rocks_column_family_handle_t,
}

impl Drop for ColumnFamilyHandle {
    fn drop(&mut self) {
        // this will not delete CF
        unsafe {
            ll::rocks_column_family_handle_destroy(self.raw);
        }
    }
}

impl AsRef<ColumnFamilyHandle> for ColumnFamilyHandle {
    fn as_ref(&self) -> &ColumnFamilyHandle {
        self
    }
}

impl fmt::Debug for ColumnFamilyHandle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CFHandle(id={}, name={:?})", self.id(), self.name())
    }
}

impl ToRaw<ll::rocks_column_family_handle_t> for ColumnFamilyHandle {
    fn raw(&self) -> *mut ll::rocks_column_family_handle_t {
        self.raw
    }
}

impl FromRaw<ll::rocks_column_family_handle_t> for ColumnFamilyHandle {
    unsafe fn from_ll(raw: *mut ll::rocks_column_family_handle_t) -> ColumnFamilyHandle {
        ColumnFamilyHandle { raw: raw }
    }
}

impl ColumnFamilyHandle {
    /// Returns the name of the column family associated with the current handle.
    pub fn name(&self) -> &str {
        unsafe {
            let ptr = ll::rocks_column_family_handle_get_name(self.raw);
            CStr::from_ptr(ptr).to_str().unwrap()
        }
    }

    /// Returns the ID of the column family associated with the current handle.
    pub fn id(&self) -> u32 {
        unsafe { ll::rocks_column_family_handle_get_id(self.raw) }
    }
}

/// An opened column family, owned for RAII style management
pub struct ColumnFamily {
    handle: ColumnFamilyHandle,
    db: Arc<DBRef>,
    owned: bool,
}

unsafe impl Sync for ColumnFamily {}
unsafe impl Send for ColumnFamily {}

impl Drop for ColumnFamily {
    fn drop(&mut self) {
        if self.owned {
            let mut status = ptr::null_mut::<ll::rocks_status_t>();
            unsafe {
                ll::rocks_db_destroy_column_family_handle(self.db.raw, self.raw(), &mut status);
                assert!(Error::from_ll(status).is_ok());
                // make underlying cf_handle a nullptr, rocks-sys will skip deleting it.
                self.handle.raw = ptr::null_mut();
            }
        }
    }
}

impl AsRef<ColumnFamilyHandle> for ColumnFamily {
    fn as_ref(&self) -> &ColumnFamilyHandle {
        &self.handle
    }
}

impl ops::Deref for ColumnFamily {
    type Target = ColumnFamilyHandle;
    fn deref(&self) -> &ColumnFamilyHandle {
        &self.handle
    }
}

impl fmt::Debug for ColumnFamily {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ColumnFamily")
            .field("db", &self.db.name())
            .field("name", &self.name())
            .field("id", &self.id())
            .finish()
    }
}

impl ColumnFamily {
    // TODO:
    // Fills "*desc" with the up-to-date descriptor of the column family
    // associated with this handle. Since it fills "*desc" with the up-to-date
    // information, this call might internally lock and release DB mutex to
    // access the up-to-date CF options.  In addition, all the pointer-typed
    // options cannot be referenced any longer than the original options exist.
    //
    // Note that this function is not supported in RocksDBLite.
    // pub fn descriptor(&self) -> Result<ColumnFamilyDescriptor> {
    // }
    //

    // Rust: migrate API from DB

    pub fn put(&self, options: &WriteOptions, key: &[u8], value: &[u8]) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_put_cf(
                self.db.raw,
                options.raw(),
                self.raw(),
                key.as_ptr() as *const _,
                key.len(),
                value.as_ptr() as *const _,
                value.len(),
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    pub fn delete(&self, options: &WriteOptions, key: &[u8]) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_delete_cf(
                self.db.raw,
                options.raw(),
                self.raw(),
                key.as_ptr() as *const _,
                key.len(),
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    pub fn single_delete(&self, options: &WriteOptions, key: &[u8]) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_single_delete_cf(
                self.db.raw,
                options.raw(),
                self.raw(),
                key.as_ptr() as *const _,
                key.len(),
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    pub fn delete_range(&self, options: &WriteOptions, begin_key: &[u8], end_key: &[u8]) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_delete_range_cf(
                self.db.raw,
                options.raw(),
                self.raw(),
                begin_key.as_ptr() as *const _,
                begin_key.len(),
                end_key.as_ptr() as *const _,
                end_key.len(),
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    pub fn merge(&self, options: &WriteOptions, key: &[u8], val: &[u8]) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_merge_cf(
                self.db.raw,
                options.raw(),
                self.raw(),
                key.as_ptr() as *const _,
                key.len(),
                val.as_ptr() as *const _,
                val.len(),
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    pub fn get(&self, options: &ReadOptions, key: &[u8]) -> Result<PinnableSlice> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        // FIXME: should be mut, should hide `new()`
        let pinnable_val = PinnableSlice::new();
        unsafe {
            ll::rocks_db_get_cf_pinnable(
                self.db.raw,
                options.raw(),
                self.raw(),
                key.as_ptr() as *const _,
                key.len(),
                pinnable_val.raw(),
                &mut status,
            );
            Error::from_ll(status).map(|_| pinnable_val)
        }
    }

    pub fn multi_get(&self, options: &ReadOptions, keys: &[&[u8]]) -> Vec<Result<PinnableSlice>> {
        let num_keys = keys.len();
        let mut statuses: Vec<*mut ll::rocks_status_t> = vec![ptr::null_mut(); num_keys];
        let mut c_values = Vec::with_capacity(num_keys);
        let values = (0..num_keys)
            .map(|_| {
                let ret = PinnableSlice::new();
                c_values.push(ret.raw());
                ret
            })
            .collect::<Vec<_>>();

        unsafe {
            ll::rocks_db_multi_get_cf_coerce(
                self.db.raw,
                options.raw(),
                num_keys,
                self.raw(),
                keys.as_ptr() as _,
                c_values.as_mut_ptr(),
                statuses.as_mut_ptr(),
            );
        }

        statuses
            .into_iter()
            .zip(values.into_iter())
            .map(|(st, val)| Error::from_ll(st).map(|_| val))
            .collect()
    }

    /// If the key definitely does not exist in the database, then this method
    /// returns false, else true. If the caller wants to obtain value when the key
    /// is found in memory, a bool for 'value_found' must be passed. 'value_found'
    /// will be true on return if value has been set properly.
    ///
    /// This check is potentially lighter-weight than invoking DB::Get(). One way
    /// to make this lighter weight is to avoid doing any IOs.
    pub fn key_may_exist(&self, options: &ReadOptions, key: &[u8]) -> bool {
        unsafe {
            ll::rocks_db_key_may_exist_cf(
                self.db.raw,
                options.raw(),
                self.raw(),
                key.as_ptr() as *const _,
                key.len(),
                ptr::null_mut(),
                ptr::null_mut(),
            ) != 0
        }
    }

    pub fn key_may_get(&self, options: &ReadOptions, key: &[u8]) -> (bool, Option<Vec<u8>>) {
        let mut found = 0;
        let mut value: Vec<u8> = vec![];
        unsafe {
            let ret = ll::rocks_db_key_may_exist_cf(
                self.db.raw,
                options.raw(),
                self.raw(),
                key.as_ptr() as *const _,
                key.len(),
                &mut value as *mut Vec<u8> as *mut c_void,
                &mut found,
            );
            if ret == 0 {
                (false, None)
            } else if found == 0 {
                (true, None)
            } else {
                (true, Some(value))
            }
        }
    }

    pub fn new_iterator(&self, options: &ReadOptions) -> Iterator {
        unsafe {
            let ptr = ll::rocks_db_create_iterator_cf(self.db.raw, options.raw(), self.raw());
            Iterator::from_ll(ptr)
        }
    }

    pub fn get_property(&self, property: &str) -> Option<String> {
        let mut ret = String::new();
        let ok = unsafe {
            ll::rocks_db_get_property_cf(
                self.db.raw,
                self.raw(),
                property.as_bytes().as_ptr() as *const _,
                property.len(),
                &mut ret as *mut String as *mut c_void,
            ) != 0
        };
        if ok {
            Some(ret)
        } else {
            None
        }
    }

    pub fn get_int_property(&self, property: &str) -> Option<u64> {
        let mut val = 0;
        let ok = unsafe {
            ll::rocks_db_get_int_property_cf(
                self.db.raw,
                self.raw(),
                property.as_bytes().as_ptr() as *const _,
                property.len(),
                &mut val,
            ) != 0
        };
        if ok {
            Some(val)
        } else {
            None
        }
    }

    pub fn compact_range<R: AsCompactRange>(&self, options: &CompactRangeOptions, range: R) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_compact_range_opt_cf(
                self.db.raw,
                options.raw(),
                self.raw(),
                range.start_key() as *const _,
                range.start_key_len(),
                range.end_key() as *const _,
                range.end_key_len(),
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    pub fn set_options<T, H>(&self, new_options: H) -> Result<()>
    where
        T: AsRef<str>,
        H: IntoIterator<Item = (T, T)>,
    {
        let mut key_ptrs = Vec::with_capacity(2);
        let mut key_lens = Vec::with_capacity(2);
        let mut val_ptrs = Vec::with_capacity(2);
        let mut val_lens = Vec::with_capacity(2);
        let num_options = new_options
            .into_iter()
            .map(|(key, val)| {
                key_ptrs.push(key.as_ref().as_ptr() as *const c_char);
                key_lens.push(key.as_ref().len());
                val_ptrs.push(val.as_ref().as_ptr() as *const c_char);
                val_lens.push(val.as_ref().len());
            })
            .count();
        let mut status = ptr::null_mut();
        unsafe {
            ll::rocks_db_set_options_cf(
                self.db.raw,
                self.raw(),
                num_options,
                key_ptrs.as_ptr(),
                key_lens.as_ptr(),
                val_ptrs.as_ptr(),
                val_lens.as_ptr(),
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    pub fn get_approximate_sizes(&self, ranges: &[ops::Range<&[u8]>]) -> Vec<u64> {
        let num_ranges = ranges.len();
        let mut range_start_ptrs = Vec::with_capacity(num_ranges);
        let mut range_start_lens = Vec::with_capacity(num_ranges);
        let mut range_end_ptrs = Vec::with_capacity(num_ranges);
        let mut range_end_lens = Vec::with_capacity(num_ranges);
        let mut sizes = vec![0_u64; num_ranges];
        for r in ranges {
            range_start_ptrs.push(r.start.as_ptr() as *const c_char);
            range_start_lens.push(r.start.len());
            range_end_ptrs.push(r.end.as_ptr() as *const c_char);
            range_end_lens.push(r.end.len());
        }
        unsafe {
            ll::rocks_db_get_approximate_sizes_cf(
                self.db.raw,
                self.raw(),
                num_ranges,
                range_start_ptrs.as_ptr(),
                range_start_lens.as_ptr(),
                range_end_ptrs.as_ptr(),
                range_end_lens.as_ptr(),
                sizes.as_mut_ptr(),
            );
        }
        sizes
    }

    pub fn get_approximate_memtable_stats(&self, range: ops::Range<&[u8]>) -> (u64, u64) {
        let mut count = 0;
        let mut size = 0;
        unsafe {
            ll::rocks_db_get_approximate_memtable_stats_cf(
                self.db.raw,
                self.raw(),
                range.start.as_ptr() as *const c_char,
                range.start.len(),
                range.end.as_ptr() as *const c_char,
                range.end.len(),
                &mut count,
                &mut size,
            );
        }
        (count, size)
    }

    pub fn ingest_external_file<P: AsRef<Path>, T: IntoIterator<Item = P>>(
        &self,
        external_files: T,
        options: &IngestExternalFileOptions,
    ) -> Result<()> {
        let mut num_files = 0;
        let mut c_files = vec![];
        let mut c_files_lens = vec![];
        for f in external_files {
            let fpath = f.as_ref().to_str().expect("valid utf8 path");
            c_files.push(fpath.as_ptr() as *const _);
            c_files_lens.push(fpath.len());
            num_files += 1;
        }
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_ingest_external_file_cf(
                self.db.raw,
                self.raw(),
                c_files.as_ptr() as *const _,
                c_files_lens.as_ptr(),
                num_files,
                options.raw(),
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    /// Obtains the meta data of the current column family of the DB.
    pub fn metadata(&self) -> ColumnFamilyMetaData {
        unsafe {
            let cfmeta = ll::rocks_db_get_column_family_metadata(self.db.raw, self.raw());

            let total_size = ll::rocks_column_family_metadata_size(cfmeta);
            let file_count = ll::rocks_column_family_metadata_file_count(cfmeta);
            let name = CStr::from_ptr(ll::rocks_column_family_metadata_name(cfmeta))
                .to_string_lossy()
                .to_owned()
                .to_string();

            let num_levels = ll::rocks_column_family_metadata_levels_count(cfmeta);

            let mut meta = ColumnFamilyMetaData {
                size: total_size,
                file_count: file_count,
                name: name,
                levels: Vec::with_capacity(num_levels as usize),
            };

            for lv in 0..num_levels {
                let level = ll::rocks_column_family_metadata_levels_level(cfmeta, lv);
                let lv_size = ll::rocks_column_family_metadata_levels_size(cfmeta, lv);

                let num_sstfiles = ll::rocks_column_family_metadata_levels_files_count(cfmeta, lv);

                // return
                let mut current_level = LevelMetaData {
                    level: level as u32,
                    size: lv_size,
                    files: Vec::with_capacity(num_sstfiles as usize),
                };

                for i in 0..num_sstfiles {
                    let name = CStr::from_ptr(ll::rocks_column_family_metadata_levels_files_name(cfmeta, lv, i))
                        .to_string_lossy()
                        .to_owned()
                        .to_string();
                    let db_path: String =
                        CStr::from_ptr(ll::rocks_column_family_metadata_levels_files_db_path(cfmeta, lv, i))
                            .to_string_lossy()
                            .to_owned()
                            .to_string();
                    let size = ll::rocks_column_family_metadata_levels_files_size(cfmeta, lv, i);

                    let small_seqno = ll::rocks_column_family_metadata_levels_files_smallest_seqno(cfmeta, lv, i);
                    let large_seqno = ll::rocks_column_family_metadata_levels_files_largest_seqno(cfmeta, lv, i);

                    let mut key_len = 0;
                    let small_key_ptr =
                        ll::rocks_column_family_metadata_levels_files_smallestkey(cfmeta, lv, i, &mut key_len);
                    let small_key = slice::from_raw_parts(small_key_ptr as *const u8, key_len).to_vec();

                    let large_key_ptr =
                        ll::rocks_column_family_metadata_levels_files_largestkey(cfmeta, lv, i, &mut key_len);
                    let large_key = slice::from_raw_parts(large_key_ptr as *const u8, key_len).to_vec();

                    let being_compacted =
                        ll::rocks_column_family_metadata_levels_files_being_compacted(cfmeta, lv, i) != 0;

                    let sst_file = SstFileMetaData {
                        size: size as u64,
                        name: name,
                        db_path: db_path,
                        smallest_seqno: small_seqno.into(),
                        largest_seqno: large_seqno.into(),
                        smallestkey: small_key,
                        largestkey: large_key,
                        being_compacted: being_compacted,
                    };

                    current_level.files.push(sst_file);
                }

                meta.levels.push(current_level);
            }

            ll::rocks_column_family_metadata_destroy(cfmeta);

            meta
        }
    }

    // ================================================================================
}

/// Borrowed DB handle
pub struct DBRef {
    raw: *mut ll::rocks_db_t,
}

impl Drop for DBRef {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            ll::rocks_db_destroy(self.raw);
        }
    }
}

impl ToRaw<ll::rocks_db_t> for DBRef {
    fn raw(&self) -> *mut ll::rocks_db_t {
        self.raw
    }
}

unsafe impl Sync for DBRef {}
unsafe impl Send for DBRef {}

/// A `DB` is a persistent ordered map from keys to values.
///
/// A `DB` is safe for concurrent access from multiple threads without
/// any external synchronization.
///
/// # Examples
///
/// ```no_run
/// use rocks::rocksdb::*;
///
/// let db = DB::open(Options::default().map_db_options(|db| db.create_if_missing(true)),
///                   "./data").unwrap();
/// // insert kv
/// let _ = db.put(&WriteOptions::default(), b"my-key", b"my-value").unwrap();
///
/// // get kv
/// let val = db.get(&ReadOptions::default(), b"my-key").unwrap();
/// println!("got value {}", String::from_utf8_lossy(&val));
///
/// assert_eq!(val, b"my-value");
/// ```
pub struct DB {
    context: Arc<DBRef>,
}

impl ops::Deref for DB {
    type Target = DBRef;

    fn deref(&self) -> &DBRef {
        &self.context
    }
}

impl fmt::Debug for DB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DB").field("name", &self.name()).finish()
    }
}

unsafe impl Sync for DB {}
unsafe impl Send for DB {}

impl ToRaw<ll::rocks_db_t> for DB {
    fn raw(&self) -> *mut ll::rocks_db_t {
        self.context.raw
    }
}

impl FromRaw<ll::rocks_db_t> for DB {
    unsafe fn from_ll(raw: *mut ll::rocks_db_t) -> DB {
        let context = DBRef { raw: raw };
        DB {
            context: Arc::new(context),
        }
    }
}

impl DB {
    /// Open the database with the specified `name`.
    pub fn open<T: AsRef<Options>, P: AsRef<Path>>(options: T, name: P) -> Result<DB> {
        let opt = options.as_ref().raw();
        let dbname = CString::new(path_to_bytes(name)).unwrap();
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            let db_ptr = ll::rocks_db_open(opt, dbname.as_ptr(), &mut status);
            Error::from_ll(status).map(|_| DB::from_ll(db_ptr))
        }
    }

    /// Open the database with the specified `name` and ttl.
    pub fn open_with_ttl<T: AsRef<Options>, P: AsRef<Path>>(options: T, name: P, ttl: Option<Duration>) -> Result<DB> {
        let opt = options.as_ref().raw();
        let dbname = CString::new(path_to_bytes(name)).unwrap();
        let ttl = ttl.map(|ttl| ttl.as_secs() as i32).unwrap_or(0);
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            let db_ptr = ll::rocks_db_open_with_ttl(opt, dbname.as_ptr(), ttl, &mut status);
            Error::from_ll(status).map(|_| DB::from_ll(db_ptr))
        }
    }

    /// Open DB with column families.
    pub fn open_with_column_families<CF: Into<ColumnFamilyDescriptor>, P: AsRef<Path>, I: IntoIterator<Item = CF>>(
        options: &DBOptions,
        name: P,
        column_families: I,
    ) -> Result<(DB, Vec<ColumnFamily>)> {
        let opt = options.raw();
        let dbname = CString::new(path_to_bytes(name)).unwrap();

        let cfs = column_families
            .into_iter()
            .map(|desc| desc.into())
            .collect::<Vec<ColumnFamilyDescriptor>>();

        let num_column_families = cfs.len();
        // for ffi
        let mut cfnames: Vec<*const c_char> = Vec::with_capacity(num_column_families);
        let mut cfopts: Vec<*const ll::rocks_cfoptions_t> = Vec::with_capacity(num_column_families);
        let mut cfhandles = vec![ptr::null_mut(); num_column_families];

        for cf in &cfs {
            cfnames.push(cf.name_as_ptr());
            cfopts.push(cf.options.raw());
        }

        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            let db_ptr = ll::rocks_db_open_column_families(
                options.raw(),
                dbname.as_ptr(),
                num_column_families as c_int,
                cfnames.as_ptr(),
                cfopts.as_ptr(),
                cfhandles.as_mut_ptr(),
                &mut status,
            );
            Error::from_ll(status).map(|_| {
                let db = DB::from_ll(db_ptr);
                let db_ref = db.context.clone();
                (
                    db,
                    cfhandles
                        .into_iter()
                        .map(|p| ColumnFamily {
                            handle: ColumnFamilyHandle { raw: p },
                            db: db_ref.clone(),
                            owned: true,
                        })
                        .collect(),
                )
            })
        }
    }

    /// Open DB with column families and ttls.
    pub fn open_with_column_families_and_ttls<
        CF: Into<ColumnFamilyDescriptor>,
        P: AsRef<Path>,
        I: IntoIterator<Item = CF>,
    >(
        options: &DBOptions,
        name: P,
        column_families: I,
        ttls: Vec<Option<Duration>>,
    ) -> Result<(DB, Vec<ColumnFamily>)> {
        let opt = options.raw();
        let dbname = CString::new(path_to_bytes(name)).unwrap();

        let cfs = column_families
            .into_iter()
            .map(|desc| desc.into())
            .collect::<Vec<ColumnFamilyDescriptor>>();

        let num_column_families = cfs.len();
        // for ffi
        let mut cfnames: Vec<*const c_char> = Vec::with_capacity(num_column_families);
        let mut cfopts: Vec<*const ll::rocks_cfoptions_t> = Vec::with_capacity(num_column_families);
        let mut cfhandles = vec![ptr::null_mut(); num_column_families];

        for cf in &cfs {
            cfnames.push(cf.name_as_ptr());
            cfopts.push(cf.options.raw());
        }

        let ttls: Vec<i32> = ttls
            .into_iter()
            .map(|opt| opt.map(|ttl| ttl.as_secs() as i32).unwrap_or(0))
            .collect();

        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            let db_ptr = ll::rocks_db_open_column_families_with_ttl(
                options.raw(),
                dbname.as_ptr(),
                num_column_families as c_int,
                cfnames.as_ptr(),
                cfopts.as_ptr(),
                cfhandles.as_mut_ptr(),
                ttls.as_ptr(),
                &mut status,
            );
            Error::from_ll(status).map(|_| {
                let db = DB::from_ll(db_ptr);
                let db_ref = db.context.clone();
                (
                    db,
                    cfhandles
                        .into_iter()
                        .map(|p| ColumnFamily {
                            handle: ColumnFamilyHandle { raw: p },
                            db: db_ref.clone(),
                            owned: true,
                        })
                        .collect(),
                )
            })
        }
    }

    /// Open the database for read only. All DB interfaces
    /// that modify data, like `put/delete`, will return error.
    /// If the db is opened in read only mode, then no compactions
    /// will happen.
    pub fn open_for_readonly<P: AsRef<Path>>(options: &Options, name: P, error_if_log_file_exist: bool) -> Result<DB> {
        let dbname = CString::new(path_to_bytes(name)).unwrap();
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            let db_ptr = ll::rocks_db_open_for_read_only(
                options.raw(),
                dbname.as_ptr(),
                error_if_log_file_exist as u8,
                &mut status,
            );
            Error::from_ll(status).map(|_| DB::from_ll(db_ptr))
        }
    }

    /// Open the database for read only with column families. When opening DB with
    /// read only, you can specify only a subset of column families in the
    /// database that should be opened. However, you always need to specify default
    /// column family. The default column family name is 'default' and it's stored
    /// in rocksdb::kDefaultColumnFamilyName
    pub fn open_for_readonly_with_column_families<
        CF: Into<ColumnFamilyDescriptor>,
        P: AsRef<Path>,
        I: IntoIterator<Item = CF>,
    >(
        options: &DBOptions,
        name: P,
        column_families: I,
        error_if_log_file_exist: bool,
    ) -> Result<(DB, Vec<ColumnFamily>)> {
        let dbname = CString::new(path_to_bytes(name)).unwrap();
        let cf_descs = column_families
            .into_iter()
            .map(|desc| desc.into())
            .collect::<Vec<ColumnFamilyDescriptor>>();

        let num_column_families = cf_descs.len();
        // for ffi
        let mut cfnames: Vec<*const c_char> = Vec::with_capacity(num_column_families);
        let mut cfopts: Vec<*const ll::rocks_cfoptions_t> = Vec::with_capacity(num_column_families);
        let mut cfhandles = vec![ptr::null_mut(); num_column_families];

        for cf_desc in &cf_descs {
            cfnames.push(cf_desc.name_as_ptr());
            cfopts.push(cf_desc.options.raw());
        }

        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            let db_ptr = ll::rocks_db_open_for_read_only_column_families(
                options.raw(),
                dbname.as_ptr(),
                num_column_families as c_int,
                cfnames.as_ptr(),
                cfopts.as_ptr(),
                cfhandles.as_mut_ptr(),
                error_if_log_file_exist as _,
                &mut status,
            );
            Error::from_ll(status).map(|_| {
                let db = DB::from_ll(db_ptr);
                let db_ref = db.context.clone();
                (
                    db,
                    cfhandles
                        .into_iter()
                        .map(|p| ColumnFamily {
                            handle: ColumnFamilyHandle { raw: p },
                            db: db_ref.clone(),
                            owned: true,
                        })
                        .collect(),
                )
            })
        }
    }

    /// Open DB as secondary instance with only the default column family.
    pub fn open_as_secondary<P1: AsRef<Path>, P2: AsRef<Path>>(
        options: &Options,
        name: P1,
        secondary_path: P2,
    ) -> Result<DB> {
        let dbname = CString::new(path_to_bytes(name)).unwrap();
        let secondary_path = CString::new(path_to_bytes(secondary_path)).unwrap();

        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            let db_ptr =
                ll::rocks_db_open_as_secondary(options.raw(), dbname.as_ptr(), secondary_path.as_ptr(), &mut status);
            Error::from_ll(status).map(|_| DB::from_ll(db_ptr))
        }
    }

    /// Open DB as secondary instance with column families. You can open a subset
    /// of column families in secondary mode.
    pub fn open_as_secondary_with_column_families<
        P1: AsRef<Path>,
        P2: AsRef<Path>,
        CF: Into<ColumnFamilyDescriptor>,
        I: IntoIterator<Item = CF>,
    >(
        dboptions: &DBOptions,
        name: P1,
        secondary_path: P2,
        column_families: I,
    ) -> Result<(DB, Vec<ColumnFamily>)> {
        let dbname = CString::new(path_to_bytes(name)).unwrap();
        let secondary_path = CString::new(path_to_bytes(secondary_path)).unwrap();
        let cf_descs = column_families
            .into_iter()
            .map(|desc| desc.into())
            .collect::<Vec<ColumnFamilyDescriptor>>();

        let num_column_families = cf_descs.len();
        // for ffi
        let mut cfnames: Vec<*const c_char> = Vec::with_capacity(num_column_families);
        let mut cfopts: Vec<*const ll::rocks_cfoptions_t> = Vec::with_capacity(num_column_families);
        let mut cfhandles = vec![ptr::null_mut(); num_column_families];

        for cf_desc in &cf_descs {
            cfnames.push(cf_desc.name_as_ptr());
            cfopts.push(cf_desc.options.raw());
        }

        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            let db_ptr = ll::rocks_db_open_as_secondary_column_families(
                dboptions.raw(),
                dbname.as_ptr(),
                secondary_path.as_ptr(),
                num_column_families as c_int,
                cfnames.as_ptr(),
                cfopts.as_ptr(),
                cfhandles.as_mut_ptr(),
                &mut status,
            );
            Error::from_ll(status).map(|_| {
                let db = DB::from_ll(db_ptr);
                let db_ref = db.context.clone();
                (
                    db,
                    cfhandles
                        .into_iter()
                        .map(|p| ColumnFamily {
                            handle: ColumnFamilyHandle { raw: p },
                            db: db_ref.clone(),
                            owned: true,
                        })
                        .collect(),
                )
            })
        }
    }

    /// `ListColumnFamilies` will open the DB specified by argument name
    /// and return the list of all column nfamilies in that DB
    /// through `column_families` argument. The ordering of
    /// column families in column_families is unspecified.
    pub fn list_column_families<P: AsRef<Path>>(options: &Options, name: P) -> Result<Vec<String>> {
        let dbname = CString::new(path_to_bytes(name)).unwrap();
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        let mut lencfs = 0;
        unsafe {
            let cfs = ll::rocks_db_list_column_families(options.raw(), dbname.as_ptr(), &mut lencfs, &mut status);
            Error::from_ll(status).map(|_| {
                if lencfs == 0 {
                    vec![]
                } else {
                    let mut ret = Vec::with_capacity(lencfs);
                    for i in 0..lencfs {
                        ret.push(CStr::from_ptr(*cfs.offset(i as isize)).to_str().unwrap().to_string());
                    }
                    ll::rocks_db_list_column_families_destroy(cfs, lencfs);
                    ret
                }
            })
        }
    }

    /// Create a column_family and return the handle of column family
    /// through the argument handle.
    pub fn create_column_family(&self, cfopts: &ColumnFamilyOptions, column_family_name: &str) -> Result<ColumnFamily> {
        let dbname = CString::new(column_family_name).unwrap();
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            let handle = ll::rocks_db_create_column_family(self.raw(), cfopts.raw(), dbname.as_ptr(), &mut status);
            Error::from_ll(status).map(|_| ColumnFamily {
                handle: ColumnFamilyHandle { raw: handle },
                db: self.context.clone(),
                owned: true,
            })
        }
    }

    /// Create a column_family with ttl and return the handle of column family
    /// through the argument handle.
    pub fn create_column_family_with_ttl(
        &self,
        cfopts: &ColumnFamilyOptions,
        column_family_name: &str,
        ttl: Option<Duration>,
    ) -> Result<ColumnFamily> {
        let dbname = CString::new(column_family_name).unwrap();
        let ttl = ttl.map(|ttl| ttl.as_secs() as i32).unwrap_or(0);
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            let handle =
                ll::rocks_db_create_column_family_with_ttl(self.raw(), cfopts.raw(), dbname.as_ptr(), ttl, &mut status);
            Error::from_ll(status).map(|_| ColumnFamily {
                handle: ColumnFamilyHandle { raw: handle },
                db: self.context.clone(),
                owned: true,
            })
        }
    }

    /// Drop a column family specified by column_family handle. This call
    /// only records a drop record in the manifest and prevents the column
    /// family from flushing and compacting.
    pub fn drop_column_family(&self, column_family: &ColumnFamilyHandle) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_drop_column_family(self.raw(), column_family.raw(), &mut status);
            Error::from_ll(status)
        }
    }

    /// Returns default column family handle
    pub fn default_column_family(&self) -> ColumnFamily {
        ColumnFamily {
            handle: ColumnFamilyHandle {
                raw: unsafe { ll::rocks_db_default_column_family(self.raw()) },
            },
            db: self.context.clone(),
            owned: false,
        }
    }
}

impl DBRef {
    /// Returns default column family handle
    fn raw_default_column_family(&self) -> *mut ll::rocks_column_family_handle_t {
        unsafe { ll::rocks_db_default_column_family(self.raw()) }
    }

    /// Close the DB by releasing resources, closing files etc. This should be
    /// called before calling the destructor so that the caller can get back a
    /// status in case there are any errors. This will not fsync the WAL files.
    /// If syncing is required, the caller must first call SyncWAL(), or Write()
    /// using an empty write batch with WriteOptions.sync=true.
    ///
    /// If the return status is Aborted(), closing fails because there is
    /// unreleased snapshot in the system. In this case, users can release
    /// the unreleased snapshots and try again and expect it to succeed. For
    /// other status, recalling Close() will be no-op.
    ///
    /// If the return status is NotSupported(), then the DB implementation does
    /// cleanup in the destructor
    ///
    /// NOTE for Rust: segmentation fault if the db is accessed after close
    pub unsafe fn close(&self) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        ll::rocks_db_close(self.raw(), &mut status);
        Error::from_ll(status)
    }

    /// Manually resume the DB and put it in read-write mode.
    /// This function will flush memtables for all the column families,
    /// clear the error, purge any obsolete files, and restart
    /// background flush and compaction operations.
    pub fn resume(&self) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_resume(self.raw(), &mut status);
        }
        Error::from_ll(status)
    }

    /// Set the database entry for `"key"` to `"value"`.
    /// If `"key"` already exists, it will be overwritten.
    /// Returns OK on success, and a non-OK status on error.
    ///
    /// Note: consider setting `options.sync = true`.
    pub fn put(&self, options: &WriteOptions, key: &[u8], value: &[u8]) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_put(
                self.raw(),
                options.raw(),
                key.as_ptr() as *const _,
                key.len(),
                value.as_ptr() as *const _,
                value.len(),
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    pub fn put_cf(
        &self,
        options: &WriteOptions,
        column_family: &ColumnFamilyHandle,
        key: &[u8],
        value: &[u8],
    ) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_put_cf(
                self.raw(),
                options.raw(),
                column_family.raw(),
                key.as_ptr() as *const _,
                key.len(),
                value.as_ptr() as *const _,
                value.len(),
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    /// Remove the database entry (if any) for "key".  Returns OK on
    /// success, and a non-OK status on error.  It is not an error if "key"
    /// did not exist in the database.
    ///
    /// Note: consider setting `options.sync = true`.
    pub fn delete(&self, options: &WriteOptions, key: &[u8]) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_delete(
                self.raw(),
                options.raw(),
                key.as_ptr() as *const _,
                key.len(),
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    pub fn delete_cf(&self, options: &WriteOptions, column_family: &ColumnFamilyHandle, key: &[u8]) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_delete_cf(
                self.raw(),
                options.raw(),
                column_family.raw(),
                key.as_ptr() as *const _,
                key.len(),
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    /// Remove the database entry for "key". Requires that the key exists
    /// and was not overwritten. Returns OK on success, and a non-OK status
    /// on error.  It is not an error if "key" did not exist in the database.
    ///
    /// If a key is overwritten (by calling Put() multiple times), then the result
    /// of calling SingleDelete() on this key is undefined.  SingleDelete() only
    /// behaves correctly if there has been only one Put() for this key since the
    /// previous call to SingleDelete() for this key.
    ///
    /// This feature is currently an experimental performance optimization
    /// for a very specific workload.  It is up to the caller to ensure that
    /// SingleDelete is only used for a key that is not deleted using Delete() or
    /// written using Merge().  Mixing SingleDelete operations with Deletes and
    /// Merges can result in undefined behavior.
    ///
    /// Note: consider setting `options.sync = true`.
    pub fn single_delete(&self, options: &WriteOptions, key: &[u8]) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_single_delete(
                self.raw(),
                options.raw(),
                key.as_ptr() as *const _,
                key.len(),
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    pub fn single_delete_cf(
        &self,
        options: &WriteOptions,
        column_family: &ColumnFamilyHandle,
        key: &[u8],
    ) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_single_delete_cf(
                self.raw(),
                options.raw(),
                column_family.raw(),
                key.as_ptr() as *const _,
                key.len(),
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    /// Removes the database entries in the range ["begin_key", "end_key"), i.e.,
    /// including "begin_key" and excluding "end_key". Returns OK on success, and
    /// a non-OK status on error. It is not an error if no keys exist in the range
    /// `["begin_key", "end_key")`.
    ///
    /// This feature is currently an experimental performance optimization for
    /// deleting very large ranges of contiguous keys. Invoking it many times or on
    /// small ranges may severely degrade read performance; in particular, the
    /// resulting performance can be worse than calling Delete() for each key in
    /// the range. Note also the degraded read performance affects keys outside the
    /// deleted ranges, and affects database operations involving scans, like flush
    /// and compaction.
    ///
    /// Consider setting `ReadOptions::ignore_range_deletions = true` to speed
    /// up reads for key(s) that are known to be unaffected by range deletions.
    pub fn delete_range_cf(
        &self,
        options: &WriteOptions,
        column_family: &ColumnFamilyHandle,
        begin_key: &[u8],
        end_key: &[u8],
    ) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_delete_range_cf(
                self.raw(),
                options.raw(),
                column_family.raw(),
                begin_key.as_ptr() as *const _,
                begin_key.len(),
                end_key.as_ptr() as *const _,
                end_key.len(),
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    /// Merge the database entry for "key" with "value".  Returns OK on success,
    /// and a non-OK status on error. The semantics of this operation is
    /// determined by the user provided merge_operator when opening DB.
    ///
    /// Note: consider setting `options.sync = true`.
    pub fn merge(&self, options: &WriteOptions, key: &[u8], val: &[u8]) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_merge(
                self.raw(),
                options.raw(),
                key.as_ptr() as *const _,
                key.len(),
                val.as_ptr() as *const _,
                val.len(),
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    pub fn merge_cf(
        &self,
        options: &WriteOptions,
        column_family: &ColumnFamilyHandle,
        key: &[u8],
        val: &[u8],
    ) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_merge_cf(
                self.raw(),
                options.raw(),
                column_family.raw(),
                key.as_ptr() as *const _,
                key.len(),
                val.as_ptr() as *const _,
                val.len(),
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    /// Apply the specified updates to the database.
    ///
    /// If `updates` contains no update, WAL will still be synced if
    /// `options.sync=true`.
    ///
    /// Returns OK on success, non-OK on failure.
    ///
    /// Note: consider setting `options.sync = true`.
    pub fn write(&self, options: &WriteOptions, updates: &WriteBatch) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_write(self.raw(), options.raw(), updates.raw(), &mut status);
            Error::from_ll(status)
        }
    }

    /// If the database contains an entry for "key" store the
    /// corresponding value in *value and return OK.
    ///
    /// If there is no entry for "key" leave *value unchanged and return
    /// a status for which Error::IsNotFound() returns true.
    ///
    /// May return some other Error on an error.
    pub fn get(&self, options: &ReadOptions, key: &[u8]) -> Result<PinnableSlice> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        // FIXME: should be mut
        let pinnable_val = PinnableSlice::new();
        unsafe {
            ll::rocks_db_get_pinnable(
                self.raw(),
                options.raw(),
                key.as_ptr() as *const _,
                key.len(),
                pinnable_val.raw(),
                &mut status,
            );
            Error::from_ll(status).map(|_| pinnable_val)
        }
    }

    pub fn get_cf(
        &self,
        options: &ReadOptions,
        column_family: &ColumnFamilyHandle,
        key: &[u8],
    ) -> Result<PinnableSlice> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        // FIXME: should be mut
        let pinnable_val = PinnableSlice::new();
        unsafe {
            ll::rocks_db_get_cf_pinnable(
                self.raw(),
                options.raw(),
                column_family.raw(),
                key.as_ptr() as _,
                key.len(),
                pinnable_val.raw(),
                &mut status,
            );
            Error::from_ll(status).map(|_| pinnable_val)
        }
    }

    /// If keys[i] does not exist in the database, then the i'th returned
    /// status will be one for which Error::IsNotFound() is true, and
    /// (*values)[i] will be set to some arbitrary value (often ""). Otherwise,
    /// the i'th returned status will have Error::ok() true, and (*values)[i]
    /// will store the value associated with keys[i].
    ///
    /// (*values) will always be resized to be the same size as (keys).
    /// Similarly, the number of returned statuses will be the number of keys.
    ///
    /// Note: keys will not be "de-duplicated". Duplicate keys will return
    /// duplicate values in order.
    pub fn multi_get(&self, options: &ReadOptions, keys: &[&[u8]]) -> Vec<Result<PinnableSlice>> {
        let num_keys = keys.len();
        let mut statuses: Vec<*mut ll::rocks_status_t> = vec![ptr::null_mut(); num_keys];
        let mut c_values = Vec::with_capacity(num_keys);
        let values = (0..num_keys)
            .map(|_| {
                let ret = PinnableSlice::new();
                c_values.push(ret.raw());
                ret
            })
            .collect::<Vec<_>>();

        unsafe {
            ll::rocks_db_multi_get_cf_coerce(
                self.raw(),
                options.raw(),
                num_keys,
                self.raw_default_column_family(),
                keys.as_ptr() as _,
                c_values.as_mut_ptr(),
                statuses.as_mut_ptr(),
            );
        }

        statuses
            .into_iter()
            .zip(values.into_iter())
            .map(|(st, val)| Error::from_ll(st).map(|_| val))
            .collect()
    }

    pub fn multi_get_cf(
        &self,
        options: &ReadOptions,
        column_families: &[&ColumnFamilyHandle],
        keys: &[&[u8]],
    ) -> Vec<Result<PinnableSlice>> {
        let num_keys = keys.len();
        let c_cfs: Vec<_> = column_families.iter().map(|cf| cf.raw() as *const _).collect();
        let mut statuses: Vec<*mut ll::rocks_status_t> = vec![ptr::null_mut(); num_keys];
        let mut c_values = Vec::with_capacity(num_keys);
        let values = (0..num_keys)
            .map(|_| {
                let ret = PinnableSlice::new();
                c_values.push(ret.raw());
                ret
            })
            .collect::<Vec<_>>();

        unsafe {
            ll::rocks_db_multi_get_cfs_coerce(
                self.raw(),
                options.raw(),
                num_keys,
                c_cfs.as_ptr(),
                keys.as_ptr() as _,
                c_values.as_mut_ptr(),
                statuses.as_mut_ptr(),
            );
        }

        statuses
            .into_iter()
            .zip(values.into_iter())
            .map(|(st, val)| Error::from_ll(st).map(|_| val))
            .collect()
    }

    /// If the key definitely does not exist in the database, then this method
    /// returns false, else true. If the caller wants to obtain value when the key
    /// is found in memory, a bool for 'value_found' must be passed. 'value_found'
    /// will be true on return if value has been set properly.
    ///
    /// This check is potentially lighter-weight than invoking DB::Get(). One way
    /// to make this lighter weight is to avoid doing any IOs.
    ///
    /// Default implementation here returns true and sets 'value_found' to false
    pub fn key_may_exist(&self, options: &ReadOptions, key: &[u8]) -> bool {
        unsafe {
            ll::rocks_db_key_may_exist(
                self.raw(),
                options.raw(),
                key.as_ptr() as *const _,
                key.len(),
                ptr::null_mut(),
                ptr::null_mut(),
            ) != 0
        }
    }

    pub fn key_may_get(&self, options: &ReadOptions, key: &[u8]) -> (bool, Option<Vec<u8>>) {
        let mut found = 0;
        let mut value: Vec<u8> = vec![];
        unsafe {
            let ret = ll::rocks_db_key_may_exist(
                self.raw(),
                options.raw(),
                key.as_ptr() as *const _,
                key.len(),
                &mut value as *mut Vec<u8> as *mut c_void,
                &mut found,
            );
            if ret == 0 {
                (false, None)
            } else if found == 0 {
                (true, None)
            } else {
                (true, Some(value))
            }
        }
    }

    pub fn key_may_exist_cf(&self, options: &ReadOptions, column_family: &ColumnFamilyHandle, key: &[u8]) -> bool {
        unsafe {
            ll::rocks_db_key_may_exist_cf(
                self.raw(),
                options.raw(),
                column_family.raw(),
                key.as_ptr() as *const _,
                key.len(),
                ptr::null_mut(),
                ptr::null_mut(),
            ) != 0
        }
    }

    pub fn key_may_get_cf(
        &self,
        options: &ReadOptions,
        column_family: &ColumnFamilyHandle,
        key: &[u8],
    ) -> (bool, Option<Vec<u8>>) {
        let mut found = 0;
        let mut value: Vec<u8> = vec![];
        unsafe {
            let ret = ll::rocks_db_key_may_exist_cf(
                self.raw(),
                options.raw(),
                column_family.raw(),
                key.as_ptr() as *const _,
                key.len(),
                &mut value as *mut Vec<u8> as *mut c_void,
                &mut found,
            );
            if ret == 0 {
                (false, None)
            } else if found == 0 {
                (true, None)
            } else {
                (true, Some(value))
            }
        }
    }

    /// Return a heap-allocated iterator over the contents of the database.
    /// The result of NewIterator() is initially invalid (caller must
    /// call one of the Seek methods on the iterator before using it).
    ///
    /// Caller should delete the iterator when it is no longer needed.
    /// The returned iterator should be deleted before this db is deleted.
    pub fn new_iterator<'c, 'd: 'c>(&'d self, options: &ReadOptions) -> Iterator<'c> {
        unsafe {
            let ptr = ll::rocks_db_create_iterator(self.raw(), options.raw());
            Iterator::from_ll(ptr)
        }
    }

    pub fn new_iterator_cf<'c, 'd: 'c>(&self, options: &ReadOptions, cf: &'d ColumnFamilyHandle) -> Iterator<'c> {
        unsafe {
            let ptr = ll::rocks_db_create_iterator_cf(self.raw(), options.raw(), cf.raw());
            Iterator::from_ll(ptr)
        }
    }

    pub fn new_iterators<'c, 'b: 'c, T: AsRef<ColumnFamilyHandle>>(
        &'b self,
        options: &ReadOptions,
        cfs: &[T],
    ) -> Result<Vec<Iterator<'c>>> {
        let c_cfs = cfs.iter().map(|cf| cf.as_ref().raw()).collect::<Vec<_>>();
        let cfs_len = cfs.len();
        let mut c_iters = vec![ptr::null_mut(); cfs_len];
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_create_iterators(
                self.raw(),
                options.raw(),
                c_cfs.as_ptr() as _,
                c_iters.as_mut_ptr(),
                cfs_len,
                &mut status,
            );
            Error::from_ll(status).map(|_| c_iters.into_iter().map(|ptr| Iterator::from_ll(ptr)).collect())
        }
    }

    /// Return a handle to the current DB state.  Iterators created with
    /// this handle will all observe a stable snapshot of the current DB
    /// state.  The caller must call ReleaseSnapshot(result) when the
    /// snapshot is no longer needed.
    ///
    /// nullptr will be returned if the DB fails to take a snapshot or does
    /// not support snapshot.
    pub fn get_snapshot(&self) -> Option<Snapshot> {
        unsafe {
            let ptr = ll::rocks_db_get_snapshot(self.raw());
            if ptr.is_null() {
                None
            } else {
                Some(Snapshot::from_ll(ptr))
            }
        }
    }

    /// Release a previously acquired snapshot.  The caller must not
    /// use "snapshot" after this call.
    pub fn release_snapshot(&self, snapshot: Snapshot) {
        unsafe {
            ll::rocks_db_release_snapshot(self.raw(), snapshot.raw());
        }
    }

    /// DB implementations can export properties about their state via this method.
    /// If "property" is a valid property understood by this DB implementation (see
    /// Properties struct above for valid options), fills "*value" with its current
    /// value and returns true.  Otherwise, returns false.
    pub fn get_property(&self, property: &str) -> Option<String> {
        let mut ret = String::new();
        let ok = unsafe {
            ll::rocks_db_get_property(
                self.raw(),
                property.as_bytes().as_ptr() as *const _,
                property.len(),
                &mut ret as *mut String as *mut c_void,
            ) != 0
        };
        if ok {
            Some(ret)
        } else {
            None
        }
    }

    pub fn get_property_cf(&self, column_family: &ColumnFamilyHandle, property: &str) -> Option<String> {
        let mut ret = String::new();
        let ok = unsafe {
            ll::rocks_db_get_property_cf(
                self.raw(),
                column_family.raw(),
                property.as_bytes().as_ptr() as *const _,
                property.len(),
                &mut ret as *mut String as *mut c_void,
            ) != 0
        };
        if ok {
            Some(ret)
        } else {
            None
        }
    }

    // TODO:
    pub fn get_map_property(&self, property: &str) -> Option<()> {
        unimplemented!()
    }

    /// Similar to `GetProperty()`, but only works for a subset of properties whose
    /// return value is an integer. Return the value by integer. Supported
    /// properties:
    ///
    /// + `"rocksdb.num-immutable-mem-table"`
    /// + `"rocksdb.mem-table-flush-pending"`
    /// + `"rocksdb.compaction-pending"`
    /// + `"rocksdb.background-errors"`
    /// + `"rocksdb.cur-size-active-mem-table"`
    /// + `"rocksdb.cur-size-all-mem-tables"`
    /// + `"rocksdb.size-all-mem-tables"`
    /// + `"rocksdb.num-entries-active-mem-table"`
    /// + `"rocksdb.num-entries-imm-mem-tables"`
    /// + `"rocksdb.num-deletes-active-mem-table"`
    /// + `"rocksdb.num-deletes-imm-mem-tables"`
    /// + `"rocksdb.estimate-num-keys"`
    /// + `"rocksdb.estimate-table-readers-mem"`
    /// + `"rocksdb.is-file-deletions-enabled"`
    /// + `"rocksdb.num-snapshots"`
    /// + `"rocksdb.oldest-snapshot-time"`
    /// + `"rocksdb.num-live-versions"`
    /// + `"rocksdb.current-super-version-number"`
    /// + `"rocksdb.estimate-live-data-size"`
    /// + `"rocksdb.min-log-number-to-keep"`
    /// + `"rocksdb.total-sst-files-size"`
    /// + `"rocksdb.base-level"`
    /// + `"rocksdb.estimate-pending-compaction-bytes"`
    /// + `"rocksdb.num-running-compactions"`
    /// + `"rocksdb.num-running-flushes"`
    /// + `"rocksdb.actual-delayed-write-rate"`
    /// + `"rocksdb.is-write-stopped"`
    pub fn get_int_property(&self, property: &str) -> Option<u64> {
        let mut val = 0;
        let ok = unsafe {
            ll::rocks_db_get_int_property(
                self.raw(),
                property.as_bytes().as_ptr() as *const _,
                property.len(),
                &mut val,
            ) != 0
        };
        if ok {
            Some(val)
        } else {
            None
        }
    }

    pub fn get_int_property_cf(&self, column_family: &ColumnFamilyHandle, property: &str) -> Option<u64> {
        let mut val = 0;
        let ok = unsafe {
            ll::rocks_db_get_int_property_cf(
                self.raw(),
                column_family.raw(),
                property.as_bytes().as_ptr() as *const _,
                property.len(),
                &mut val,
            ) != 0
        };
        if ok {
            Some(val)
        } else {
            None
        }
    }

    /// Same as GetIntProperty(), but this one returns the aggregated int
    /// property from all column families.
    pub fn get_aggregated_int_property(&self, property: &str) -> Option<u64> {
        let mut val = 0;
        let ok = unsafe {
            ll::rocks_db_get_aggregated_int_property(
                self.raw(),
                property.as_bytes().as_ptr() as *const _,
                property.len(),
                &mut val,
            ) != 0
        };
        if ok {
            Some(val)
        } else {
            None
        }
    }

    pub fn get_approximate_sizes(&self, column_family: &ColumnFamilyHandle, ranges: &[ops::Range<&[u8]>]) -> Vec<u64> {
        // include_flags: u8
        let num_ranges = ranges.len();
        let mut range_start_ptrs = Vec::with_capacity(num_ranges);
        let mut range_start_lens = Vec::with_capacity(num_ranges);
        let mut range_end_ptrs = Vec::with_capacity(num_ranges);
        let mut range_end_lens = Vec::with_capacity(num_ranges);
        let mut sizes = vec![0_u64; num_ranges];
        for r in ranges {
            range_start_ptrs.push(r.start.as_ptr() as *const c_char);
            range_start_lens.push(r.start.len());
            range_end_ptrs.push(r.end.as_ptr() as *const c_char);
            range_end_lens.push(r.end.len());
        }
        unsafe {
            ll::rocks_db_get_approximate_sizes_cf(
                self.raw(),
                column_family.raw(),
                num_ranges,
                range_start_ptrs.as_ptr(),
                range_start_lens.as_ptr(),
                range_end_ptrs.as_ptr(),
                range_end_lens.as_ptr(),
                sizes.as_mut_ptr(),
            );
        }
        sizes
    }

    pub fn get_approximate_memtable_stats(
        &self,
        column_family: &ColumnFamilyHandle,
        range: ops::Range<&[u8]>,
    ) -> (u64, u64) {
        let mut count = 0;
        let mut size = 0;
        unsafe {
            ll::rocks_db_get_approximate_memtable_stats_cf(
                self.raw(),
                column_family.raw(),
                range.start.as_ptr() as *const c_char,
                range.start.len(),
                range.end.as_ptr() as *const c_char,
                range.end.len(),
                &mut count,
                &mut size,
            );
        }
        (count, size)
    }

    /// Compact the underlying storage for the key range `[*begin,*end]`.
    /// The actual compaction interval might be superset of `[*begin, *end]`.
    /// In particular, deleted and overwritten versions are discarded,
    /// and the data is rearranged to reduce the cost of operations
    /// needed to access the data.  This operation should typically only
    /// be invoked by users who understand the underlying implementation.
    ///
    /// `begin==nullptr` is treated as a key before all keys in the database.
    /// `end==nullptr` is treated as a key after all keys in the database.
    /// Therefore the following call will compact the entire database:
    ///
    /// > `db->CompactRange(options, nullptr, nullptr);`
    ///
    /// Note that after the entire database is compacted, all data are pushed
    /// down to the last level containing any data. If the total data size after
    /// compaction is reduced, that level might not be appropriate for hosting all
    /// the files. In this case, client could set options.change_level to true, to
    /// move the files back to the minimum level capable of holding the data set
    /// or a given level (specified by non-negative options.target_level).
    ///
    /// For Rust: use range expr, and since `compact_range()` use superset of range.
    pub fn compact_range<R: AsCompactRange>(&self, options: &CompactRangeOptions, range: R) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_compact_range_opt(
                self.raw(),
                options.raw(),
                range.start_key() as *const _,
                range.start_key_len(),
                range.end_key() as *const _,
                range.end_key_len(),
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    pub fn set_options<T, H>(&self, column_family: &ColumnFamilyHandle, new_options: H) -> Result<()>
    where
        T: AsRef<str>,
        H: IntoIterator<Item = (T, T)>,
    {
        let mut key_ptrs = Vec::with_capacity(2);
        let mut key_lens = Vec::with_capacity(2);
        let mut val_ptrs = Vec::with_capacity(2);
        let mut val_lens = Vec::with_capacity(2);
        let num_options = new_options
            .into_iter()
            .map(|(key, val)| {
                key_ptrs.push(key.as_ref().as_ptr() as *const c_char);
                key_lens.push(key.as_ref().len());
                val_ptrs.push(val.as_ref().as_ptr() as *const c_char);
                val_lens.push(val.as_ref().len());
            })
            .count();
        let mut status = ptr::null_mut();
        unsafe {
            ll::rocks_db_set_options_cf(
                self.raw(),
                column_family.raw,
                num_options,
                key_ptrs.as_ptr(),
                key_lens.as_ptr(),
                val_ptrs.as_ptr(),
                val_lens.as_ptr(),
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    pub fn set_db_options(&self, new_options: &HashMap<&str, &str>) -> Result<()> {
        let num_options = new_options.len();
        let mut key_ptrs = Vec::with_capacity(num_options);
        let mut key_lens = Vec::with_capacity(num_options);
        let mut val_ptrs = Vec::with_capacity(num_options);
        let mut val_lens = Vec::with_capacity(num_options);
        new_options
            .iter()
            .map(|(key, val)| {
                key_ptrs.push(key.as_ptr() as *const c_char);
                key_lens.push(key.len());
                val_ptrs.push(val.as_ptr() as *const c_char);
                val_lens.push(val.len());
            })
            .last();
        let mut status = ptr::null_mut();
        unsafe {
            ll::rocks_db_set_db_options(
                self.raw(),
                num_options,
                key_ptrs.as_ptr(),
                key_lens.as_ptr(),
                val_ptrs.as_ptr(),
                val_lens.as_ptr(),
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    /// CompactFiles() inputs a list of files specified by file numbers and
    /// compacts them to the specified level. Note that the behavior is different
    /// from CompactRange() in that CompactFiles() performs the compaction job
    /// using the CURRENT thread.
    pub fn compact_files<P: AsRef<Path>, I: IntoIterator<Item = P>>(
        &self,
        compact_options: &CompactionOptions,
        input_file_names: I,
        output_level: i32,
    ) -> Result<()> {
        self.compact_files_to(compact_options, input_file_names, output_level, -1)
    }

    pub fn compact_files_to<P: AsRef<Path>, I: IntoIterator<Item = P>>(
        &self,
        compact_options: &CompactionOptions,
        input_file_names: I,
        output_level: i32,
        output_path_id: i32,
    ) -> Result<()> {
        let mut c_file_names = Vec::new();
        let mut c_file_name_sizes = Vec::new();
        for file_name in input_file_names {
            let file_path = file_name.as_ref().to_str().unwrap();
            c_file_names.push(file_path.as_bytes().as_ptr() as *const _);
            c_file_name_sizes.push(file_path.len());
        }
        let mut status = ptr::null_mut();
        unsafe {
            ll::rocks_db_compact_files(
                self.raw(),
                compact_options.raw(),
                c_file_names.len(),
                c_file_names.as_ptr(),
                c_file_name_sizes.as_ptr(),
                output_level as c_int,
                output_path_id as c_int,
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    /// This function will wait until all currently running background processes
    /// finish. After it returns, no background process will be run until
    /// ContinueBackgroundWork is called
    pub fn pause_background_work(&self) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_pause_background_work(self.raw(), &mut status);
            Error::from_ll(status)
        }
    }

    pub fn continue_background_work(&self) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_continue_background_work(self.raw(), &mut status);
            Error::from_ll(status)
        }
    }

    /// Request stopping background work, if wait is true wait until it's done
    pub fn cancel_background_work(&self, wait: bool) {
        unsafe {
            ll::rocks_cancel_all_background_work(self.raw(), wait as u8);
        }
    }

    /// This function will enable automatic compactions for the given column
    /// families if they were previously disabled. The function will first set the
    /// disable_auto_compactions option for each column family to 'false', after
    /// which it will schedule a flush/compaction.
    ///
    /// NOTE: Setting disable_auto_compactions to 'false' through SetOptions() API
    /// does NOT schedule a flush/compaction afterwards, and only changes the
    /// parameter itself within the column family option.
    pub fn enable_auto_compaction(&self, column_family_handles: &[&ColumnFamilyHandle]) -> Result<()> {
        let c_cfs = column_family_handles
            .iter()
            .map(|cf| cf.as_ref().raw() as *const _)
            .collect::<Vec<*const _>>();
        let cfs_len = column_family_handles.len();
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_enable_auto_compaction(self.raw(), c_cfs.as_ptr(), cfs_len, &mut status);
            Error::from_ll(status)
        }
    }

    /// Number of levels used for this DB.
    pub fn number_levels(&self) -> u32 {
        unsafe { ll::rocks_db_number_levels(self.raw()) as u32 }
    }

    /// Maximum level to which a new compacted memtable is pushed if it
    /// does not create overlap.
    pub fn max_mem_compaction_level(&self) -> u32 {
        unsafe { ll::rocks_db_max_mem_compaction_level(self.raw()) as u32 }
    }

    /// Number of files in level-0 that would stop writes.
    pub fn level0_stop_write_trigger(&self) -> u32 {
        unsafe { ll::rocks_db_level0_stop_write_trigger(self.raw()) as u32 }
    }

    /// Get DB name -- the exact same name that was provided as an argument to
    /// `DB::Open()`
    pub fn name(&self) -> String {
        let mut name = String::new();
        unsafe {
            ll::rocks_db_get_name(self.raw(), &mut name as *mut String as *mut c_void);
        }
        name
    }

    // TODO:
    // get options
    // get db options

    /// Flush all mem-table data.
    pub fn flush(&self, options: &FlushOptions) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_flush(self.raw(), options.raw(), &mut status);
            Error::from_ll(status)
        }
    }

    /// Sync the wal. Note that Write() followed by SyncWAL() is not exactly the
    /// same as Write() with sync=true: in the latter case the changes won't be
    /// visible until the sync is done.
    ///
    /// Currently only works if allow_mmap_writes = false in Options.
    pub fn sync_wal(&self) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_sync_wal(self.raw(), &mut status);
            Error::from_ll(status)
        }
    }

    /// The sequence number of the most recent transaction.
    pub fn get_latest_sequence_number(&self) -> SequenceNumber {
        unsafe { ll::rocks_db_get_latest_sequence_number(self.raw()).into() }
    }

    /// Prevent file deletions. Compactions will continue to occur,
    /// but no obsolete files will be deleted. Calling this multiple
    /// times have the same effect as calling it once.
    pub fn disable_file_deletions(&self) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_disable_file_deletions(self.raw(), &mut status);
            Error::from_ll(status)
        }
    }

    /// Allow compactions to delete obsolete files.
    ///
    /// If `force == true`, the call to EnableFileDeletions() will guarantee that
    /// file deletions are enabled after the call, even if DisableFileDeletions()
    /// was called multiple times before.
    ///
    /// If `force == false`, EnableFileDeletions will only enable file deletion
    /// after it's been called at least as many times as DisableFileDeletions(),
    /// enabling the two methods to be called by two threads concurrently without
    /// synchronization -- i.e., file deletions will be enabled only after both
    /// threads call EnableFileDeletions()
    pub fn enable_file_deletions(&self, force: bool) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_enable_file_deletions(self.raw(), force as u8, &mut status);
            Error::from_ll(status)
        }
    }

    /// GetLiveFiles followed by GetSortedWalFiles can generate a lossless backup
    ///
    /// Retrieve the list of all files in the database. The files are
    /// relative to the dbname and are not absolute paths. The valid size of the
    /// manifest file is returned in manifest_file_size. The manifest file is an
    /// ever growing file, but only the portion specified by manifest_file_size is
    /// valid for this snapshot.
    /// Setting flush_memtable to true does Flush before recording the live files.
    /// Setting flush_memtable to false is useful when we don't want to wait for
    /// flush which may have to wait for compaction to complete taking an
    /// indeterminate time.
    ///
    /// In case you have multiple column families, even if flush_memtable is true,
    /// you still need to call GetSortedWalFiles after GetLiveFiles to compensate
    /// for new data that arrived to already-flushed column families while other
    /// column families were flushing
    pub fn get_live_files(&self, flush_memtable: bool) -> Result<(u64, Vec<String>)> {
        let mut manifest_file_size = 0;
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            let files =
                ll::rocks_db_get_live_files(self.raw(), flush_memtable as u8, &mut manifest_file_size, &mut status);
            Error::from_ll(status).map(|_| {
                let n = ll::cxx_string_vector_size(files) as usize;
                let mut ret = Vec::with_capacity(n);
                for i in 0..n {
                    let f = slice::from_raw_parts(
                        ll::cxx_string_vector_nth(files, i) as *const u8,
                        ll::cxx_string_vector_nth_size(files, i),
                    );
                    ret.push(String::from_utf8_lossy(f).to_owned().to_string());
                }
                ll::cxx_string_vector_destory(files);
                (manifest_file_size, ret)
            })
        }
    }

    /// Retrieve the sorted list of all wal files with earliest file first
    pub fn get_sorted_wal_files(&self) -> Result<Vec<LogFile>> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            let cfiles = ll::rocks_db_get_sorted_wal_files(self.raw(), &mut status);
            Error::from_ll(status).map(|()| {
                let num_files = ll::rocks_logfiles_size(cfiles);
                let mut files = Vec::with_capacity(num_files);
                for i in 0..num_files {
                    let mut path_name = String::new();
                    ll::rocks_logfiles_nth_path_name(cfiles, i, &mut path_name as *mut String as *mut c_void);
                    let log_num = ll::rocks_logfiles_nth_log_number(cfiles, i);
                    let file_type = mem::transmute(ll::rocks_logfiles_nth_type(cfiles, i));
                    let start_seq = ll::rocks_logfiles_nth_start_sequence(cfiles, i);
                    let file_size = ll::rocks_logfiles_nth_file_size(cfiles, i);
                    files.push(LogFile {
                        path_name: path_name,
                        log_number: log_num,
                        file_type: file_type,
                        start_sequence: start_seq.into(),
                        size_in_bytes: file_size,
                    })
                }
                ll::rocks_logfiles_destroy(cfiles);
                files
            })
        }
    }

    /// Sets iter to an iterator that is positioned at a write-batch containing
    /// seq_number. If the sequence number is non existent, it returns an iterator
    /// at the first available seq_no after the requested seq_no
    ///
    /// Returns Error::OK if iterator is valid
    ///
    /// Must set WAL_ttl_seconds or WAL_size_limit_MB to large values to
    /// use this api, else the WAL files will get
    /// cleared aggressively and the iterator might keep getting invalid before
    /// an update is read.
    pub fn get_updates_since(&self, seq_number: SequenceNumber) -> Result<TransactionLogIterator> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            let iter_raw_ptr = ll::rocks_db_get_update_since(self.raw(), seq_number.0, &mut status);
            Error::from_ll(status).map(|_| TransactionLogIterator::from_ll(iter_raw_ptr))
        }
    }

    /// Delete the file name from the db directory and update the internal state to
    /// reflect that. Supports deletion of sst and log files only. 'name' must be
    /// path relative to the db directory. eg. 000001.sst, /archive/000003.log
    pub fn delete_file(&self, name: &str) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_delete_file(
                self.raw(),
                name.as_bytes().as_ptr() as *const _,
                name.len(),
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    /// Delete files which are entirely in the given range
    ///
    /// Could leave some keys in the range which are in files which are not
    /// entirely in the range.
    ///
    /// Snapshots before the delete might not see the data in the given range.
    pub fn delete_files_in_range(&self, column_family: &ColumnFamilyHandle, begin: &[u8], end: &[u8]) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_delete_files_in_range(
                self.raw(),
                column_family.raw(),
                begin.as_ptr() as *const _,
                begin.len(),
                end.as_ptr() as *const _,
                end.len(),
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    /// Returns a list of all table files with their level, start key
    /// and end key
    pub fn get_live_files_metadata(&self) -> Vec<LiveFileMetaData> {
        unsafe {
            let livefiles = ll::rocks_db_get_livefiles_metadata(self.raw());

            let cnt = ll::rocks_livefiles_count(livefiles);
            let mut ret = Vec::with_capacity(cnt as usize);
            for i in 0..cnt {
                let name = CStr::from_ptr(ll::rocks_livefiles_name(livefiles, i))
                    .to_string_lossy()
                    .to_owned()
                    .to_string();
                let db_path: String = CStr::from_ptr(ll::rocks_livefiles_db_path(livefiles, i))
                    .to_string_lossy()
                    .to_owned()
                    .to_string();
                let size = ll::rocks_livefiles_size(livefiles, i);

                let small_seqno = ll::rocks_livefiles_smallest_seqno(livefiles, i);
                let large_seqno = ll::rocks_livefiles_largest_seqno(livefiles, i);

                let mut key_len = 0;
                let small_key_ptr = ll::rocks_livefiles_smallestkey(livefiles, i, &mut key_len);
                let small_key = slice::from_raw_parts(small_key_ptr as *const u8, key_len).to_vec();

                let large_key_ptr = ll::rocks_livefiles_largestkey(livefiles, i, &mut key_len);
                let large_key = slice::from_raw_parts(large_key_ptr as *const u8, key_len).to_vec();

                let being_compacted = ll::rocks_livefiles_being_compacted(livefiles, i) != 0;

                let cf_name = CStr::from_ptr(ll::rocks_livefiles_column_family_name(livefiles, i))
                    .to_string_lossy()
                    .to_owned()
                    .to_string();
                let level = ll::rocks_livefiles_level(livefiles, i);

                let meta = LiveFileMetaData {
                    sst_file: SstFileMetaData {
                        size: size as u64,
                        name: name,
                        db_path: db_path,
                        smallest_seqno: small_seqno.into(),
                        largest_seqno: large_seqno.into(),
                        smallestkey: small_key,
                        largestkey: large_key,
                        being_compacted: being_compacted,
                    },
                    column_family_name: cf_name,
                    level: level as u32,
                };

                ret.push(meta);
            }
            ll::rocks_livefiles_destroy(livefiles);
            ret
        }
    }

    /// Obtains the meta data of the specified column family of the DB.
    pub fn get_column_family_metadata(&self, column_family: &ColumnFamilyHandle) -> ColumnFamilyMetaData {
        unsafe {
            let cfmeta = ll::rocks_db_get_column_family_metadata(self.raw(), column_family.raw());

            let total_size = ll::rocks_column_family_metadata_size(cfmeta);
            let file_count = ll::rocks_column_family_metadata_file_count(cfmeta);
            let name = CStr::from_ptr(ll::rocks_column_family_metadata_name(cfmeta))
                .to_string_lossy()
                .to_owned()
                .to_string();

            let num_levels = ll::rocks_column_family_metadata_levels_count(cfmeta);

            let mut meta = ColumnFamilyMetaData {
                size: total_size,
                file_count: file_count,
                name: name,
                levels: Vec::with_capacity(num_levels as usize),
            };

            for lv in 0..num_levels {
                let level = ll::rocks_column_family_metadata_levels_level(cfmeta, lv);
                let lv_size = ll::rocks_column_family_metadata_levels_size(cfmeta, lv);

                let num_sstfiles = ll::rocks_column_family_metadata_levels_files_count(cfmeta, lv);

                let mut current_level = LevelMetaData {
                    level: level as u32,
                    size: lv_size,
                    files: Vec::with_capacity(num_sstfiles as usize),
                };

                for i in 0..num_sstfiles {
                    let name = CStr::from_ptr(ll::rocks_column_family_metadata_levels_files_name(cfmeta, lv, i))
                        .to_string_lossy()
                        .to_owned()
                        .to_string();
                    let db_path: String =
                        CStr::from_ptr(ll::rocks_column_family_metadata_levels_files_db_path(cfmeta, lv, i))
                            .to_string_lossy()
                            .to_owned()
                            .to_string();
                    let size = ll::rocks_column_family_metadata_levels_files_size(cfmeta, lv, i);

                    let small_seqno = ll::rocks_column_family_metadata_levels_files_smallest_seqno(cfmeta, lv, i);
                    let large_seqno = ll::rocks_column_family_metadata_levels_files_largest_seqno(cfmeta, lv, i);

                    let mut key_len = 0;
                    let small_key_ptr =
                        ll::rocks_column_family_metadata_levels_files_smallestkey(cfmeta, lv, i, &mut key_len);
                    let small_key = slice::from_raw_parts(small_key_ptr as *const u8, key_len).to_vec();

                    let large_key_ptr =
                        ll::rocks_column_family_metadata_levels_files_largestkey(cfmeta, lv, i, &mut key_len);
                    let large_key = slice::from_raw_parts(large_key_ptr as *const u8, key_len).to_vec();

                    let being_compacted =
                        ll::rocks_column_family_metadata_levels_files_being_compacted(cfmeta, lv, i) != 0;

                    let sst_file = SstFileMetaData {
                        size: size as u64,
                        name: name,
                        db_path: db_path,
                        smallest_seqno: small_seqno.into(),
                        largest_seqno: large_seqno.into(),
                        smallestkey: small_key,
                        largestkey: large_key,
                        being_compacted: being_compacted,
                    };

                    current_level.files.push(sst_file);
                }

                meta.levels.push(current_level);
            }

            ll::rocks_column_family_metadata_destroy(cfmeta);

            meta
        }
    }

    /// `IngestExternalFile()` will load a list of external SST files (1) into the DB
    /// We will try to find the lowest possible level that the file can fit in, and
    /// ingest the file into this level (2). A file that have a key range that
    /// overlap with the memtable key range will require us to Flush the memtable
    /// first before ingesting the file.
    ///
    /// - External SST files can be created using SstFileWriter
    /// - We will try to ingest the files to the lowest possible level even if the file compression
    ///   dont match the level compression
    pub fn ingest_external_file<P: AsRef<Path>, T: IntoIterator<Item = P>>(
        &self,
        external_files: T,
        options: &IngestExternalFileOptions,
    ) -> Result<()> {
        let mut num_files = 0;
        let mut c_files = vec![];
        let mut c_files_lens = vec![];
        for f in external_files {
            let fpath = f.as_ref().to_str().expect("valid utf8 path");
            c_files.push(fpath.as_ptr() as *const _);
            c_files_lens.push(fpath.len());
            num_files += 1;
        }
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_ingest_external_file(
                self.raw(),
                c_files.as_ptr() as *const _,
                c_files_lens.as_ptr(),
                num_files,
                options.raw(),
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    pub fn ingest_external_file_cf<P: AsRef<Path>, T: IntoIterator<Item = P>>(
        &self,
        column_family: &ColumnFamilyHandle,
        external_files: T,
        options: &IngestExternalFileOptions,
    ) -> Result<()> {
        let mut num_files = 0;
        let mut c_files = vec![];
        let mut c_files_lens = vec![];
        for f in external_files {
            let fpath = f.as_ref().to_str().expect("valid utf8 path");
            c_files.push(fpath.as_ptr() as *const _);
            c_files_lens.push(fpath.len());
            num_files += 1;
        }
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_ingest_external_file_cf(
                self.raw(),
                column_family.raw,
                c_files.as_ptr() as *const _,
                c_files_lens.as_ptr(),
                num_files,
                options.raw(),
                &mut status,
            );
            Error::from_ll(status)
        }
    }

    /// Sets the globally unique ID created at database creation time by invoking
    /// `Env::GenerateUniqueId()`, in identity. Returns Error::OK if identity could
    /// be set properly
    pub fn get_db_identity(&self) -> Result<String> {
        let mut identity = String::new();
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_get_db_identity(self.raw(), &mut identity as *mut String as *mut _, &mut status);
            Error::from_ll(status).map(|_| identity)
        }
    }

    pub fn get_properties_of_all_tables_cf(
        &self,
        column_family: &ColumnFamilyHandle,
    ) -> Result<TablePropertiesCollection> {
        let mut status = ptr::null_mut();
        unsafe {
            let props_ptr = ll::rocks_db_get_properties_of_all_tables(self.raw(), column_family.raw, &mut status);
            Error::from_ll(status).map(|()| TablePropertiesCollection::from_ll(props_ptr))
        }
    }

    pub fn get_properties_of_tables_in_range(
        &self,
        column_family: &ColumnFamilyHandle,
        ranges: &[ops::Range<&[u8]>],
    ) -> Result<TablePropertiesCollection> {
        let mut status = ptr::null_mut();
        let num_ranges = ranges.len();
        let mut start_keys = Vec::with_capacity(num_ranges);
        let mut start_key_lens = Vec::with_capacity(num_ranges);
        let mut limit_keys = Vec::with_capacity(num_ranges);
        let mut limit_key_lens = Vec::with_capacity(num_ranges);
        for r in ranges {
            start_keys.push(r.start.as_ptr() as *const c_char);
            start_key_lens.push(r.start.len());
            limit_keys.push(r.end.as_ptr() as *const c_char);
            limit_key_lens.push(r.end.len());
        }
        unsafe {
            let props_ptr = ll::rocks_db_get_properties_of_tables_in_range(
                self.raw(),
                column_family.raw,
                num_ranges,
                start_keys.as_ptr(),
                start_key_lens.as_ptr(),
                limit_keys.as_ptr(),
                limit_key_lens.as_ptr(),
                &mut status,
            );
            Error::from_ll(status).map(|()| TablePropertiesCollection::from_ll(props_ptr))
        }
    }

    // debug
    /// Returns listing of all versions of keys in the provided user key range.
    /// The range is inclusive-inclusive, i.e., [`begin_key`, `end_key`].
    /// The result is inserted into the provided vector, `key_versions`.
    pub fn get_all_key_versions(&self, begin_key: &[u8], end_key: &[u8]) -> Result<KeyVersionVec> {
        let mut status = ptr::null_mut();
        unsafe {
            let coll_ptr = ll::rocks_db_get_all_key_versions(
                self.raw(),
                begin_key.as_ptr() as *const _,
                begin_key.len(),
                end_key.as_ptr() as *const _,
                end_key.len(),
                &mut status,
            );
            Error::from_ll(status).map(|()| KeyVersionVec::from_ll(coll_ptr))
        }
    }

    /// Make the secondary instance catch up with the primary by tailing and
    /// replaying the MANIFEST and WAL of the primary.
    ///
    /// Column families created by the primary after the secondary instance starts
    /// will be ignored unless the secondary instance closes and restarts with the
    /// newly created column families.
    ///
    /// Column families that exist before secondary instance starts and dropped by
    /// the primary afterwards will be marked as dropped. However, as long as the
    /// secondary instance does not delete the corresponding column family
    /// handles, the data of the column family is still accessible to the
    /// secondary.
    pub fn try_catch_up_with_primary(&self) -> Result<()> {
        let mut status = ptr::null_mut();
        unsafe {
            ll::rocks_db_try_catch_up_with_primary(self.raw(), &mut status);
        }
        Error::from_ll(status)
    }

    /*
    // utilities/info_log_finder.h
    /// This function can be used to list the Information logs,
    /// given the db pointer.
    pub fn get_info_log_list(&self) -> Result<Vec<String>> {
        let mut status = ptr::null_mut();
        unsafe {
            let cvec = ll::rocks_db_get_info_log_list(self.raw(), &mut status);
            Error::from_ll(status).map(|()| {
                let size = ll::cxx_string_vector_size(cvec);
                let ret = (0..size).into_iter()
                    .map(|i| {
                        let base = ll::cxx_string_vector_nth(cvec, i) as *const u8;
                        let len = ll::cxx_string_vector_nth_size(cvec, i);
                        str::from_utf8_unchecked(slice::from_raw_parts(base, len)).into()
                    })
                    .collect();
                ll::cxx_string_vector_destory(cvec);
                ret
                })
        }
    }
    */
}

// ==================================================

// public functions

/// Destroy the contents of the specified database.
///
/// Be very careful using this method.
pub fn destroy_db<P: AsRef<Path>>(options: &Options, name: P) -> Result<()> {
    let name = name.as_ref().to_str().expect("valid utf8");
    let mut status = ptr::null_mut();
    unsafe {
        ll::rocks_destroy_db(options.raw(), name.as_ptr() as *const _, name.len(), &mut status);
        Error::from_ll(status)
    }
}

/// If a DB cannot be opened, you may attempt to call this method to
/// resurrect as much of the contents of the database as possible.
/// Some data may be lost, so be careful when calling this function
/// on a database that contains important information.
///
/// With this API, we will warn and skip data associated with column families not
/// specified in `column_families`.
///
/// `column_families` Descriptors for known column families
pub fn repair_db_with_cf<P: AsRef<Path>>(
    db_options: &DBOptions,
    dbname: P,
    column_families: &[&ColumnFamilyDescriptor],
) -> Result<()> {
    unimplemented!()
}

/// `unknown_cf_opts` Options for column families encountered during the
/// repair that were not specified in `column_families`.
pub fn repair_db_with_unknown_cf_opts<P: AsRef<Path>>(
    db_options: &DBOptions,
    dbname: P,
    column_families: &[&ColumnFamilyDescriptor],
    unknown_cf_opts: &ColumnFamilyOptions,
) -> Result<()> {
    unimplemented!()
}

/// `options` These options will be used for the database and for ALL column
/// families encountered during the repair.
pub fn repair_db<P: AsRef<Path>>(options: &Options, name: P) -> Result<()> {
    let name = name.as_ref().to_str().expect("valid utf8");
    let mut status = ptr::null_mut();
    unsafe {
        ll::rocks_repair_db(options.raw(), name.as_ptr() as *const _, name.len(), &mut status);
        Error::from_ll(status)
    }
}

pub trait AsCompactRange {
    fn start_key(&self) -> *const u8 {
        ptr::null()
    }

    fn start_key_len(&self) -> usize {
        0
    }

    fn end_key(&self) -> *const u8 {
        ptr::null()
    }

    fn end_key_len(&self) -> usize {
        0
    }
}

impl<'a> AsCompactRange for ops::RangeInclusive<&'a [u8]> {
    fn start_key(&self) -> *const u8 {
        self.start().as_ptr()
    }

    fn start_key_len(&self) -> usize {
        self.start().len()
    }

    fn end_key(&self) -> *const u8 {
        self.end().as_ptr()
    }

    fn end_key_len(&self) -> usize {
        self.end().len()
    }
}

impl<'a> AsCompactRange for ops::RangeToInclusive<&'a [u8]> {
    fn end_key(&self) -> *const u8 {
        self.end.as_ptr()
    }

    fn end_key_len(&self) -> usize {
        self.end.len()
    }
}

impl<'a> AsCompactRange for ops::RangeFrom<&'a [u8]> {
    fn start_key(&self) -> *const u8 {
        self.start.as_ptr()
    }

    fn start_key_len(&self) -> usize {
        self.start.len()
    }
}

impl AsCompactRange for ops::RangeFull {}
