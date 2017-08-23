//! A DB is a persistent ordered map from keys to values.

use std::mem;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::str;
use std::slice;
use std::ops;
use std::fmt;
use std::iter::IntoIterator;
use std::marker::PhantomData;
use std::path::Path;
use std::collections::hash_map::HashMap;
use std::borrow::Borrow;

use rocks_sys as ll;

use error::Status;
use options::{ColumnFamilyOptions, CompactRangeOptions, CompactionOptions, DBOptions, FlushOptions,
              IngestExternalFileOptions, Options, ReadOptions, WriteOptions};
use table_properties::TablePropertiesCollection;
use snapshot::Snapshot;
use write_batch::WriteBatch;
use iterator::Iterator;
use types::SequenceNumber;
use to_raw::{FromRaw, ToRaw};
use metadata::{ColumnFamilyMetaData, LevelMetaData, LiveFileMetaData, SstFileMetaData};
use transaction_log::{LogFile, TransactionLogIterator};
use debug::KeyVersionVec;

use super::Result;
use super::slice::{CVec, PinnableSlice};

const DEFAULT_COLUMN_FAMILY_NAME: &'static str = "default";

pub struct ColumnFamilyDescriptor {
    name: CString,
    options: Option<ColumnFamilyOptions>,
}

impl ColumnFamilyDescriptor {
    fn with_name(name: &str) -> ColumnFamilyDescriptor {
        ColumnFamilyDescriptor {
            name: CString::new(name).expect("need a valid column family name"),
            options: None,
        }
    }

    fn name_as_ptr(&self) -> *const c_char {
        self.name.as_ptr()
    }

    pub fn new(name: &str, options: ColumnFamilyOptions) -> ColumnFamilyDescriptor {
        ColumnFamilyDescriptor {
            name: CString::new(name).expect("need a valid column family name"),
            options: Some(options),
        }
    }
}

// FIXME: default column family uses default ColumnFamilyOptions
impl Default for ColumnFamilyDescriptor {
    fn default() -> Self {
        ColumnFamilyDescriptor::new(DEFAULT_COLUMN_FAMILY_NAME, ColumnFamilyOptions::default())
    }
}

impl From<String> for ColumnFamilyDescriptor {
    fn from(name: String) -> Self {
        ColumnFamilyDescriptor::with_name(&name)
    }
}

impl<'a> From<&'a str> for ColumnFamilyDescriptor {
    fn from(name: &'a str) -> Self {
        ColumnFamilyDescriptor::with_name(name)
    }
}

impl<'a> From<(&'a str, ColumnFamilyOptions)> for ColumnFamilyDescriptor {
    fn from((name, options): (&'a str, ColumnFamilyOptions)) -> Self {
        ColumnFamilyDescriptor::new(name, options)
    }
}

/// Handle for a opened column family
pub struct ColumnFamilyHandle<'a, 'b: 'a> {
    raw: *mut ll::rocks_column_family_handle_t,
    db: &'a DBRef<'b>,
    owned: bool,
}

impl<'a, 'b> Drop for ColumnFamilyHandle<'a, 'b> {
    fn drop(&mut self) {
        if self.owned {
            let mut status = ptr::null_mut::<ll::rocks_status_t>();
            unsafe {
                ll::rocks_db_destroy_column_family_handle(self.db.raw, self.raw(), &mut status);
                assert!(Status::from_ll(status).is_ok());
            }
        } else {
            unsafe {
                // this will not delete default CF
                ll::rocks_column_family_handle_destroy(self.raw);
            }
        }
    }
}

// FIXME: is this right?
impl<'a, 'b> AsRef<ColumnFamilyHandle<'a, 'b>> for ColumnFamilyHandle<'a, 'b> {
    fn as_ref(&self) -> &ColumnFamilyHandle<'a, 'b> {
        self
    }
}

impl<'a, 'b> fmt::Debug for ColumnFamilyHandle<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CFHandle(id={}, name={:?})", self.id(), self.name())
    }
}

impl<'a, 'b> ToRaw<ll::rocks_column_family_handle_t> for ColumnFamilyHandle<'a, 'b> {
    fn raw(&self) -> *mut ll::rocks_column_family_handle_t {
        self.raw
    }
}

impl<'a, 'b: 'a> ColumnFamilyHandle<'a, 'b> {
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

    // TODO:
    // Returns the comparator of the column family associated with the
    // current handle.
    // pub fn get_comparator(&self) -> Comparator {
    // unimplemented!()
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
            Status::from_ll(status)
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
            Status::from_ll(status)
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
            Status::from_ll(status)
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
            Status::from_ll(status)
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
            Status::from_ll(status)
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
            Status::from_ll(status).map(|_| pinnable_val)
        }
    }

    pub fn multi_get(&self, options: &ReadOptions, keys: &[&[u8]]) -> Vec<Result<CVec<u8>>> {
        let num_keys = keys.len();
        let mut c_keys: Vec<*const c_char> = Vec::with_capacity(num_keys);
        let mut c_keys_lens = Vec::with_capacity(num_keys);

        let c_cfs = vec![self.raw() as *const _; num_keys];

        let mut vals = vec![ptr::null_mut(); num_keys];
        let mut vals_lens = vec![0_usize; num_keys];

        for key in keys {
            c_keys.push(key.as_ptr() as *const c_char);
            c_keys_lens.push(key.len());
        }

        let mut status: Vec<*mut ll::rocks_status_t> = vec![ptr::null_mut(); num_keys];
        let mut ret = Vec::with_capacity(num_keys);
        unsafe {
            ll::rocks_db_multi_get_cf(
                self.db.raw,
                options.raw(),
                c_cfs.as_ptr(),
                num_keys,
                c_keys.as_ptr(),
                c_keys_lens.as_ptr(),
                vals.as_mut_ptr(),
                vals_lens.as_mut_ptr(),
                status.as_mut_ptr(),
            );

            for i in 0..num_keys {
                ret.push(Status::from_ll(status[i]).map(|_| CVec::from_raw_parts(vals[i] as *mut u8, vals_lens[i])));
            }
            ret
        }
    }

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
        if ok { Some(ret) } else { None }
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
        if ok { Some(val) } else { None }
    }

    pub fn compact_range<R: ToCompactRange>(&self, options: &CompactRangeOptions, range: R) -> Result<()> {
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
            Status::from_ll(status)
        }
    }

    pub fn set_options(&self, new_options: &HashMap<&str, &str>) -> Result<()> {
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
            Status::from_ll(status)
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
            Status::from_ll(status)
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
                    let db_path: String = CStr::from_ptr(
                        ll::rocks_column_family_metadata_levels_files_db_path(cfmeta, lv, i),
                    ).to_string_lossy()
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
pub struct DBRef<'a> {
    raw: *mut ll::rocks_db_t,
    _marker: PhantomData<&'a ()>,
}

impl<'a> ToRaw<ll::rocks_db_t> for DBRef<'a> {
    fn raw(&self) -> *mut ll::rocks_db_t {
        self.raw
    }
}

unsafe impl<'a> Sync for DBRef<'a> {}
unsafe impl<'a> Send for DBRef<'a> {}

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
pub struct DB<'a> {
    context: Box<DBRef<'a>>,
}

impl<'a> Drop for DB<'a> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            ll::rocks_db_close(self.context.raw());
        }
    }
}

impl<'a> Borrow<DBRef<'a>> for DB<'a> {
    fn borrow(&self) -> &DBRef<'a> {
        &self.context
    }
}

impl<'a> ops::Deref for DB<'a> {
    type Target = DBRef<'a>;

    fn deref(&self) -> &DBRef<'a> {
        &self.context
    }
}

impl<'a> fmt::Debug for DB<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DB({:?})", self.name())
    }
}

unsafe impl<'a> Sync for DB<'a> {}
unsafe impl<'a> Send for DB<'a> {}

impl<'a> ToRaw<ll::rocks_db_t> for DB<'a> {
    fn raw(&self) -> *mut ll::rocks_db_t {
        self.context.raw
    }
}

impl<'a> FromRaw<ll::rocks_db_t> for DB<'a> {
    unsafe fn from_ll(raw: *mut ll::rocks_db_t) -> DB<'a> {
        let context = DBRef {
            raw: raw,
            _marker: PhantomData,
        };
        DB { context: Box::new(context) }
    }
}

impl<'a> DB<'a> {
    /// Open the database with the specified `name`.
    pub fn open<'b, T: AsRef<Options>, P: AsRef<Path>>(options: T, name: P) -> Result<DB<'b>> {
        let opt = options.as_ref().raw();
        let dbname = name.as_ref()
            .to_str()
            .and_then(|s| CString::new(s).ok())
            .unwrap();
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            let db_ptr = ll::rocks_db_open(opt, dbname.as_ptr(), &mut status);
            Status::from_ll(status).map(|_| DB::from_ll(db_ptr))
        }
    }

    /// Open DB with column families.
    ///
    /// `db_options` specify database specific options
    ///
    /// `column_families` is the vector of all column families in the database,
    /// containing column family name and options. You need to open ALL column
    /// families in the database. To get the list of column families, you can use
    /// ListColumnFamilies(). Also, you can open only a subset of column families
    /// for read-only access.
    ///
    /// The default column family name is `'default'` and it's stored
    /// in `rocksdb::kDefaultColumnFamilyName`.
    ///
    /// If everything is OK, handles will on return be the same size
    /// as `column_families` --- `handles[i]` will be a handle that you
    /// will use to operate on column family `column_family[i]`.
    ///
    // FIXME: this should be DBOptions
    // FIXME: lifetime leaks
    pub fn open_with_column_families<
        'b: 'c,
        'c,
        CF: Into<ColumnFamilyDescriptor>,
        P: AsRef<Path>,
        I: IntoIterator<Item = CF>,
    >(
        options: &Options,
        name: P,
        column_families: I,
    ) -> Result<(DB<'b>, Vec<ColumnFamilyHandle<'c, 'b>>)> {
        let opt = options.raw();
        let dbname = name.as_ref()
            .to_str()
            .and_then(|s| CString::new(s).ok())
            .unwrap();

        let cfs = column_families
            .into_iter()
            .map(|desc| desc.into())
            .collect::<Vec<ColumnFamilyDescriptor>>();

        let num_column_families = cfs.len();
        // for ffi
        let mut cfnames: Vec<*const c_char> = Vec::with_capacity(num_column_families);
        let mut cfopts: Vec<*const ll::rocks_cfoptions_t> = Vec::with_capacity(num_column_families);
        let mut cfhandles = vec![ptr::null_mut(); num_column_families];

        // FIXME: is it necessary to create?
        // hold it to call ffi function
        let default_cfopt = ColumnFamilyOptions::from_options(options);

        for cf in &cfs {
            cfnames.push(cf.name_as_ptr());
            cfopts.push(cf.options.as_ref().unwrap_or_else(|| &default_cfopt).raw());
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
            Status::from_ll(status).map(|_| {
                let db = DB::from_ll(db_ptr);
                // lifetime transmute
                let db_ref = mem::transmute(&*db.context);
                (
                    db,
                    cfhandles
                        .into_iter()
                        .map(|p| {
                            ColumnFamilyHandle {
                                raw: p,
                                db: db_ref,
                                owned: true,
                            }
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
    pub fn open_for_readonly<'b, P: AsRef<Path>>(
        options: &Options,
        name: P,
        error_if_log_file_exist: bool,
    ) -> Result<DB<'b>> {
        let dbname = name.as_ref()
            .to_str()
            .and_then(|s| CString::new(s).ok())
            .unwrap();
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            let db_ptr = ll::rocks_db_open_for_read_only(
                options.raw(),
                dbname.as_ptr(),
                error_if_log_file_exist as u8,
                &mut status,
            );
            Status::from_ll(status).map(|_| DB::from_ll(db_ptr))
        }
    }


    /// `ListColumnFamilies` will open the DB specified by argument name
    /// and return the list of all column nfamilies in that DB
    /// through `column_families` argument. The ordering of
    /// column families in column_families is unspecified.
    pub fn list_column_families<P: AsRef<Path>>(options: &Options, name: P) -> Result<Vec<String>> {
        let dbname = name.as_ref()
            .to_str()
            .and_then(|s| CString::new(s).ok())
            .unwrap();
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        let mut lencfs = 0;
        unsafe {
            let cfs = ll::rocks_db_list_column_families(options.raw(), dbname.as_ptr(), &mut lencfs, &mut status);
            Status::from_ll(status).map(|_| if lencfs == 0 {
                vec![]
            } else {
                let mut ret = Vec::with_capacity(lencfs);
                for i in 0..lencfs {
                    ret.push(
                        CStr::from_ptr(*cfs.offset(i as isize))
                            .to_str()
                            .unwrap()
                            .to_string(),
                    );
                }
                ll::rocks_db_list_column_families_destroy(cfs, lencfs);
                ret
            })
        }
    }
}

impl<'a> DBRef<'a> {
    /// Create a column_family and return the handle of column family
    /// through the argument handle.
    pub fn create_column_family(
        &self,
        cfopts: &ColumnFamilyOptions,
        column_family_name: &str,
    ) -> Result<ColumnFamilyHandle> {
        let dbname = CString::new(column_family_name).unwrap();
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            let handle = ll::rocks_db_create_column_family(self.raw(), cfopts.raw(), dbname.as_ptr(), &mut status);
            Status::from_ll(status).map(|_| {
                ColumnFamilyHandle {
                    raw: handle,
                    db: self.borrow(),
                    owned: true,
                }
            })
        }
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
            Status::from_ll(status)
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
            Status::from_ll(status)
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
            ll::rocks_db_delete(self.raw(), options.raw(), key.as_ptr() as *const _, key.len(), &mut status);
            Status::from_ll(status)
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
            Status::from_ll(status)
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
            ll::rocks_db_single_delete(self.raw(), options.raw(), key.as_ptr() as *const _, key.len(), &mut status);
            Status::from_ll(status)
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
            Status::from_ll(status)
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
            Status::from_ll(status)
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
            Status::from_ll(status)
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
            Status::from_ll(status)
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
    pub fn write(&self, options: &WriteOptions, updates: WriteBatch) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_write(self.raw(), options.raw(), updates.raw(), &mut status);
            Status::from_ll(status)
        }
    }

    /// If the database contains an entry for "key" store the
    /// corresponding value in *value and return OK.
    ///
    /// If there is no entry for "key" leave *value unchanged and return
    /// a status for which Status::IsNotFound() returns true.
    ///
    /// May return some other Status on an error.
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
            Status::from_ll(status).map(|_| pinnable_val)
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
            Status::from_ll(status).map(|_| pinnable_val)
        }
    }

    /// If keys[i] does not exist in the database, then the i'th returned
    /// status will be one for which Status::IsNotFound() is true, and
    /// (*values)[i] will be set to some arbitrary value (often ""). Otherwise,
    /// the i'th returned status will have Status::ok() true, and (*values)[i]
    /// will store the value associated with keys[i].
    ///
    /// (*values) will always be resized to be the same size as (keys).
    /// Similarly, the number of returned statuses will be the number of keys.
    ///
    /// Note: keys will not be "de-duplicated". Duplicate keys will return
    /// duplicate values in order.
    pub fn multi_get(&self, options: &ReadOptions, keys: &[&[u8]]) -> Vec<Result<CVec<u8>>> {
        let num_keys = keys.len();
        let mut c_keys: Vec<*const c_char> = Vec::with_capacity(num_keys);
        let mut c_keys_lens = Vec::with_capacity(num_keys);

        let mut vals = vec![ptr::null_mut(); num_keys];
        let mut vals_lens = vec![0_usize; num_keys];

        for key in keys {
            c_keys.push(key.as_ptr() as *const c_char);
            c_keys_lens.push(key.len());
        }

        let mut status: Vec<*mut ll::rocks_status_t> = vec![ptr::null_mut(); num_keys];
        let mut ret = Vec::with_capacity(num_keys);

        unsafe {
            ll::rocks_db_multi_get(
                self.raw(),
                options.raw(),
                num_keys,
                c_keys.as_ptr(),
                c_keys_lens.as_ptr(),
                vals.as_mut_ptr(),
                vals_lens.as_mut_ptr(),
                status.as_mut_ptr(),
            );

            for i in 0..num_keys {
                ret.push(Status::from_ll(status[i]).map(|_| CVec::from_raw_parts(vals[i] as *mut u8, vals_lens[i])));
            }
            ret
        }
    }

    pub fn multi_get_cf(
        &self,
        options: &ReadOptions,
        column_families: &[&ColumnFamilyHandle],
        keys: &[&[u8]],
    ) -> Vec<Result<CVec<u8>>> {
        let num_keys = keys.len();
        let mut c_keys: Vec<*const c_char> = Vec::with_capacity(num_keys);
        let mut c_keys_lens = Vec::with_capacity(num_keys);
        let mut c_cfs = Vec::with_capacity(num_keys);

        let mut vals = vec![ptr::null_mut(); num_keys];
        let mut vals_lens = vec![0_usize; num_keys];

        for i in 0..num_keys {
            c_keys.push(keys[i].as_ptr() as *const c_char);
            c_keys_lens.push(keys[i].len());
            c_cfs.push(column_families[i].raw() as *const _);
        }

        let mut status: Vec<*mut ll::rocks_status_t> = vec![ptr::null_mut(); num_keys];
        let mut ret = Vec::with_capacity(num_keys);

        unsafe {
            ll::rocks_db_multi_get_cf(
                self.raw(),
                options.raw(),
                c_cfs.as_ptr(),
                num_keys,
                c_keys.as_ptr(),
                c_keys_lens.as_ptr(),
                vals.as_mut_ptr(),
                vals_lens.as_mut_ptr(),
                status.as_mut_ptr(),
            );

            for i in 0..num_keys {
                ret.push(Status::from_ll(status[i]).map(|_| CVec::from_raw_parts(vals[i] as *mut u8, vals_lens[i])));
            }
            ret
        }
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

    pub fn new_iterators<'c, 'b: 'c, T: AsRef<ColumnFamilyHandle<'c, 'b>>>(
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
            Status::from_ll(status).map(|_| {
                c_iters
                    .into_iter()
                    .map(|ptr| Iterator::from_ll(ptr))
                    .collect()
            })
        }
    }

    /// Return a handle to the current DB state.  Iterators created with
    /// this handle will all observe a stable snapshot of the current DB
    /// state.  The caller must call ReleaseSnapshot(result) when the
    /// snapshot is no longer needed.
    ///
    /// nullptr will be returned if the DB fails to take a snapshot or does
    /// not support snapshot.
    pub fn get_snapshot(&'a self) -> Option<Snapshot<'a>> {
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
        if ok { Some(ret) } else { None }
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
        if ok { Some(ret) } else { None }
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
        if ok { Some(val) } else { None }
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
        if ok { Some(val) } else { None }
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
        if ok { Some(val) } else { None }
    }

    /// For each i in [0,n-1], store in "sizes[i]", the approximate
    /// file system space used by keys in "[range[i].start .. range[i].limit)".
    ///
    /// Note that the returned sizes measure file system space usage, so
    /// if the user data compresses by a factor of ten, the returned
    /// sizes will be one-tenth the size of the corresponding user data size.
    ///
    /// If include_flags defines whether the returned size should include
    /// the recently written data in the mem-tables (if
    /// the mem-table type supports it), data serialized to disk, or both.
    /// include_flags should be of type DB::SizeApproximationFlags
    pub fn get_approximate_sizes(&self, ranges: &[ops::Range<&[u8]>]) -> Vec<u64> {
        self.get_approximate_sizes_cf(&self.default_column_family(), ranges)
    }

    pub fn get_approximate_sizes_cf(
        &self,
        column_family: &ColumnFamilyHandle,
        ranges: &[ops::Range<&[u8]>],
    ) -> Vec<u64> {
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
                column_family.raw,
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

    /// The method is similar to GetApproximateSizes, except it
    /// returns approximate number of records in memtables.
    pub fn get_approximate_memtable_stats(&self, range: ops::Range<&[u8]>) -> (u64, u64) {
        self.get_approximate_memtable_stats_cf(&self.default_column_family(), range)
    }

    pub fn get_approximate_memtable_stats_cf(
        &self,
        column_family: &ColumnFamilyHandle,
        range: ops::Range<&[u8]>,
    ) -> (u64, u64) {
        let mut count = 0;
        let mut size = 0;
        unsafe {
            ll::rocks_db_get_approximate_memtable_stats_cf(
                self.raw(),
                column_family.raw,
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
    /// For Rust: use range expr, and since `compact_range()` use superset of range,
    /// we ignore inclusive relation.
    pub fn compact_range<R: ToCompactRange>(&self, options: &CompactRangeOptions, range: R) -> Result<()> {
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
            Status::from_ll(status)
        }
    }

    pub fn set_options(&self, new_options: &HashMap<&str, &str>) -> Result<()> {
        self.set_options_cf(&self.default_column_family(), new_options)
    }

    pub fn set_options_cf(&self, column_family: &ColumnFamilyHandle, new_options: &HashMap<&str, &str>) -> Result<()> {
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
            Status::from_ll(status)
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
            Status::from_ll(status)
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
            Status::from_ll(status)
        }
    }


    /// This function will wait until all currently running background processes
    /// finish. After it returns, no background process will be run until
    /// UnblockBackgroundWork is called
    pub fn pause_background_work(&self) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_pause_background_work(self.raw(), &mut status);
            Status::from_ll(status)
        }
    }

    pub fn continue_background_work(&self) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_continue_background_work(self.raw(), &mut status);
            Status::from_ll(status)
        }
    }

    /// Request stopping background work, if wait is true wait until it's done
    ///
    /// Original in rocksdb/utilities/convenience.h
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
            Status::from_ll(status)
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
            Status::from_ll(status)
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
            Status::from_ll(status)
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
            Status::from_ll(status)
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
            Status::from_ll(status)
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
        let mut file_size = 0;
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            let files = ll::rocks_db_get_live_files(self.raw(), flush_memtable as u8, &mut file_size, &mut status);
            Status::from_ll(status).map(|_| {
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
                (file_size, ret)
            })
        }
    }

    /// Retrieve the sorted list of all wal files with earliest file first
    pub fn get_sorted_wal_files(&self) -> Result<Vec<LogFile>> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            let cfiles = ll::rocks_db_get_sorted_wal_files(self.raw(), &mut status);
            Status::from_ll(status).map(|()| {
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
    /// Returns Status::OK if iterator is valid
    ///
    /// Must set WAL_ttl_seconds or WAL_size_limit_MB to large values to
    /// use this api, else the WAL files will get
    /// cleared aggressively and the iterator might keep getting invalid before
    /// an update is read.
    pub fn get_updates_since(&self, seq_number: SequenceNumber) -> Result<TransactionLogIterator> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            let iter_raw_ptr = ll::rocks_db_get_update_since(self.raw(), seq_number.0, &mut status);
            Status::from_ll(status).map(|_| TransactionLogIterator::from_ll(iter_raw_ptr))
        }
    }

    /// Delete the file name from the db directory and update the internal state to
    /// reflect that. Supports deletion of sst and log files only. 'name' must be
    /// path relative to the db directory. eg. 000001.sst, /archive/000003.log
    pub fn delete_file(&self, name: &str) -> Result<()> {
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_delete_file(self.raw(), name.as_bytes().as_ptr() as *const _, name.len(), &mut status);
            Status::from_ll(status)
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
            Status::from_ll(status)
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
                    let db_path: String = CStr::from_ptr(
                        ll::rocks_column_family_metadata_levels_files_db_path(cfmeta, lv, i),
                    ).to_string_lossy()
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
    /// - We will try to ingest the files to the lowest possible level
    ///   even if the file compression dont match the level compression
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
            Status::from_ll(status)
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
            Status::from_ll(status)
        }
    }

    /// Sets the globally unique ID created at database creation time by invoking
    /// `Env::GenerateUniqueId()`, in identity. Returns Status::OK if identity could
    /// be set properly
    pub fn get_db_identity(&self) -> Result<String> {
        let mut identity = String::new();
        let mut status = ptr::null_mut::<ll::rocks_status_t>();
        unsafe {
            ll::rocks_db_get_db_identity(self.raw(), &mut identity as *mut String as *mut _, &mut status);
            Status::from_ll(status).map(|_| identity)
        }
    }

    /// Returns default column family handle
    pub fn default_column_family(&self) -> ColumnFamilyHandle {
        ColumnFamilyHandle {
            raw: unsafe { ll::rocks_db_default_column_family(self.raw()) },
            db: self.borrow(),
            owned: false,
        }
    }

    pub fn get_properties_of_all_tables_cf(
        &self,
        column_family: &ColumnFamilyHandle,
    ) -> Result<TablePropertiesCollection> {
        let mut status = ptr::null_mut();
        unsafe {
            let props_ptr = ll::rocks_db_get_properties_of_all_tables(self.raw(), column_family.raw, &mut status);
            Status::from_ll(status).map(|()| TablePropertiesCollection::from_ll(props_ptr))
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
            Status::from_ll(status).map(|()| TablePropertiesCollection::from_ll(props_ptr))
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
            Status::from_ll(status).map(|()| KeyVersionVec::from_ll(coll_ptr))
        }
    }

    // utilities/info_log_finder.h
    /// This function can be used to list the Information logs,
    /// given the db pointer.
    pub fn get_info_log_list(&self) -> Result<Vec<String>> {
        let mut status = ptr::null_mut();
        unsafe {
            let cvec = ll::rocks_db_get_info_log_list(self.raw(), &mut status);
            Status::from_ll(status).map(|()| {
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
}

// ==================================================

// public functions

/// Destroy the contents of the specified database.
///
/// Be very careful using this method.
pub fn destroy_db(name: &str, options: &Options) -> Result<()> {
    unimplemented!()
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
pub fn repair_db(dbname: &str, db_options: &DBOptions, column_families: &[&ColumnFamilyDescriptor]) -> Result<()> {
    unimplemented!()
}

/// `unknown_cf_opts` Options for column families encountered during the
/// repair that were not specified in `column_families`.
pub fn repair_db_with_unknown_cf_opts(
    dbname: &str,
    db_options: &DBOptions,
    column_families: &[&ColumnFamilyDescriptor],
    unknown_cf_opts: &ColumnFamilyOptions,
) -> Result<()> {
    unimplemented!()
}

/// `options` These options will be used for the database and for ALL column
/// families encountered during the repair.
pub fn repair_db_all_cfs(dbname: &str, options: &Options) -> Result<()> {
    unimplemented!()
}


// TODO: reimpl with std::collections::range::RangeArgument
pub trait ToCompactRange {
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

impl<'a> ToCompactRange for ops::Range<&'a [u8]> {
    fn start_key(&self) -> *const u8 {
        self.start.as_ptr()
    }

    fn start_key_len(&self) -> usize {
        self.start.len()
    }

    fn end_key(&self) -> *const u8 {
        self.end.as_ptr()
    }

    fn end_key_len(&self) -> usize {
        self.end.len()
    }
}

impl<'a> ToCompactRange for ops::RangeTo<&'a [u8]> {
    fn end_key(&self) -> *const u8 {
        self.end.as_ptr()
    }

    fn end_key_len(&self) -> usize {
        self.end.len()
    }
}

impl<'a> ToCompactRange for ops::RangeFrom<&'a [u8]> {
    fn start_key(&self) -> *const u8 {
        self.start.as_ptr()
    }

    fn start_key_len(&self) -> usize {
        self.start.len()
    }
}

impl ToCompactRange for ops::RangeFull {}

#[test]
fn it_works() {
    use super::advanced_options::CompactionPri;
    use tempdir::TempDir;

    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();
    let path = tmp_dir.path().to_str().unwrap();

    // staircase style config
    let opt = Options::default()
        .map_db_options(|dbopt| dbopt.create_if_missing(true))
        .map_cf_options(|cfopt| cfopt.compaction_pri(CompactionPri::MinOverlappingRatio))
        .optimize_for_small_db();
    let db = DB::open(&opt, path);
    assert!(db.is_ok(), "err => {:?}", db);
    let db = db.unwrap();
    let cfhandle = db.create_column_family(&ColumnFamilyOptions::default(), "lock");
    println!("cf => {:?}", cfhandle);

    assert!(db.name().contains("rocks"));

    assert!(db.get_info_log_list().is_ok());
    assert!(db.get_info_log_list().unwrap().contains(&"LOG".to_string()));
}

#[test]
fn test_open_for_readonly() {
    use tempdir::TempDir;

    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();
    let path = tmp_dir.path().to_str().unwrap();

    {
        let opt = Options::default().map_db_options(|opt| opt.create_if_missing(true));
        let db = DB::open(&opt, path);
        assert!(db.is_ok());
    }

    let db = DB::open_for_readonly(&Options::default(), path, false);
    assert!(db.is_ok());
}


#[test]
fn test_list_cfs() {
    use tempdir::TempDir;

    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();
    let path = tmp_dir.path().to_str().unwrap();

    {
        let opt = Options::default().map_db_options(|opt| opt.create_if_missing(true));
        let db = DB::open(&opt, path);
        assert!(db.is_ok());

        let db = db.unwrap();
        let ret = db.create_column_family(&ColumnFamilyOptions::default(), "cf1");
        assert!(ret.is_ok());

        let ret = db.create_column_family(&ColumnFamilyOptions::default(), "cf2");
        assert!(ret.is_ok());
    }

    let opt = Options::default();
    let ret = DB::list_column_families(&opt, path);
    assert!(ret.is_ok());
    assert!(ret.as_ref().unwrap().contains(&"default".to_owned()));
    assert!(ret.as_ref().unwrap().contains(&"cf1".to_owned()));
    assert!(ret.as_ref().unwrap().contains(&"cf2".to_owned()));

    let cfs = ret.unwrap();
    if let Ok((db, cf_handles)) = DB::open_with_column_families(&Options::default(), path, cfs) {
        let iters = db.new_iterators(&ReadOptions::default(), &cf_handles);
        assert!(iters.is_ok());
    }

}

#[test]
fn test_db_get() {
    use tempdir::TempDir;

    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();
    let path = tmp_dir.path().to_str().unwrap();

    {

        let opt = Options::default().map_db_options(|dbopt| dbopt.create_if_missing(true));

        let db = DB::open(&opt, path);
        assert!(db.is_ok(), "err => {:?}", db.as_ref().unwrap_err());
        let db = db.unwrap();
        let _ = db.put(&WriteOptions::default(), b"name", b"BH1XUW");
    }

    let db = DB::open(Options::default(), path).unwrap();
    let val = db.get(&ReadOptions::default(), b"name");
    assert_eq!(val.unwrap().as_ref(), b"BH1XUW");
}


#[test]
fn test_open_cf() {
    use tempdir::TempDir;
    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();

    let opt = Options::default().map_db_options(|db| db.create_if_missing(true));

    let ret =
        DB::open_with_column_families(&opt, tmp_dir.path().to_str().unwrap(), vec![ColumnFamilyDescriptor::default()]);
    assert!(ret.is_ok(), "err => {:?}", ret);
    println!("cfs => {:?}", ret);

    if let Ok((db, cfs)) = ret {
        let cf = &cfs[0];
        println!("cf name => {:?} id => {}", cf.name(), cf.id());
    }
}


#[test]
#[ignore]
// FIXME: lifetime leaks
fn test_cf_lifetime() {
    use tempdir::TempDir;
    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();

    let opt = Options::default().map_db_options(|db| db.create_if_missing(true));

    let mut cf_handle = None;
    {
        let ret = DB::open_with_column_families(
            &opt,
            tmp_dir.path().to_str().unwrap(),
            vec![ColumnFamilyDescriptor::default()],
        );
        assert!(ret.is_ok(), "err => {:?}", ret);
        println!("cfs => {:?}", ret);

        if let Ok((db, mut cfs)) = ret {
            let cf = cfs.pop().unwrap();
            println!("cf name => {:?} id => {}", cf.name(), cf.id());
            cf_handle = Some(cf);
        }
    }
    println!("cf name => {:?}", cf_handle.unwrap().name());
}


#[test]
fn test_key_may_exist() {
    use tempdir::TempDir;
    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();

    let db = DB::open(Options::default().map_db_options(|db| db.create_if_missing(true)), tmp_dir).unwrap();

    db.put(&WriteOptions::default(), b"name", b"value").unwrap();

    assert!(db.key_may_exist(&ReadOptions::default(), b"name"));
    assert!(!db.key_may_exist(&ReadOptions::default(), b"name2"))
}


#[test]
fn test_ingest_sst_file() {
    use sst_file_writer::SstFileWriter;

    let sst_dir = ::tempdir::TempDir::new_in(".", "rocks.sst").unwrap();

    let writer = SstFileWriter::builder().build();
    writer.open(sst_dir.path().join("2333.sst")).unwrap();
    for i in 0..999 {
        let key = format!("B{:05}", i);
        let value = format!("ABCDEFGH{:03}IJKLMN", i);
        writer.put(key.as_bytes(), value.as_bytes()).unwrap();
    }
    let info = writer.finish().unwrap();
    assert_eq!(info.num_entries(), 999);

    let tmp_db_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();

    let db = DB::open(Options::default().map_db_options(|db| db.create_if_missing(true)), &tmp_db_dir).unwrap();

    let ret = db.ingest_external_file(&[sst_dir.path().join("2333.sst")], &IngestExternalFileOptions::default());
    assert!(ret.is_ok(), "ingest external file: {:?}", ret);

    assert!(db.get(&ReadOptions::default(), b"B00000").is_ok());
    assert_eq!(db.get(&ReadOptions::default(), b"B00000").unwrap(), b"ABCDEFGH000IJKLMN");
    assert_eq!(db.get(&ReadOptions::default(), b"B00998").unwrap(), b"ABCDEFGH998IJKLMN");
    assert!(db.get(&ReadOptions::default(), b"B00999").is_err());

    drop(sst_dir);
    drop(tmp_db_dir);
}

#[cfg(test)]
mod tests {
    use std::iter;
    use super::*;
    use super::super::rocksdb::*;

    #[test]
    fn compact_range() {
        let s = b"123123123";
        let e = b"asdfasfasfasf";

        let _: ::std::ops::Range<&[u8]> = s.as_ref()..e.as_ref();

        let tmp_db_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();

        let opt = Options::default().map_db_options(|dbopt| dbopt.create_if_missing(true));

        let db = DB::open(opt, &tmp_db_dir).unwrap();

        let _ = db.put(&WriteOptions::default(), b"name", b"BH1XUW")
            .unwrap();
        for i in 0..100 {
            let key = format!("test2-key-{}", i);
            let val = format!("rocksdb-value-{}", i * 10);
            let value: String = iter::repeat(val).take(1000).collect::<Vec<_>>().concat();

            db.put(&WriteOptions::default(), key.as_bytes(), value.as_bytes())
                .unwrap();
        }

        // will be shown in LOG file
        let ret = db.compact_range(&CompactRangeOptions::default(), b"test2-key-5".as_ref()..b"test2-key-9".as_ref());
        assert!(ret.is_ok());

        let ret = db.compact_range(&CompactRangeOptions::default(), ..);
        assert!(ret.is_ok());

        drop(tmp_db_dir);
    }

    #[test]
    fn multi_get() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(Options::default().map_db_options(|db| db.create_if_missing(true)), &tmp_dir).unwrap();

        assert!(db.put(&Default::default(), b"a", b"1").is_ok());
        assert!(db.put(&Default::default(), b"b", b"2").is_ok());
        assert!(db.put(&Default::default(), b"c", b"3").is_ok());
        assert!(
            db.put(&Default::default(), b"long-key", b"long-value")
                .is_ok()
        );
        assert!(db.put(&Default::default(), b"e", b"5").is_ok());
        assert!(db.put(&Default::default(), b"f", b"6").is_ok());

        assert!(db.compact_range(&Default::default(), ..).is_ok());

        let ret = db.multi_get(&ReadOptions::default(), &[b"a", b"b", b"c", b"f", b"long-key", b"non-exist"]);

        assert_eq!(ret[0].as_ref().unwrap(), b"1".as_ref());
        assert_eq!(ret[1].as_ref().unwrap(), b"2".as_ref());
        assert_eq!(ret[2].as_ref().unwrap(), b"3".as_ref());
        assert_eq!(ret[3].as_ref().unwrap(), b"6".as_ref());
        assert_eq!(ret[4].as_ref().unwrap(), b"long-value".as_ref());
        assert!(ret[5].as_ref().unwrap_err().is_not_found());
    }

    #[test]
    fn multi_get_cf() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(Options::default().map_db_options(|db| db.create_if_missing(true)), &tmp_dir).unwrap();

        let def = db.default_column_family();
        let cf1 = db.create_column_family(&Default::default(), "db1").unwrap();
        let cf2 = db.create_column_family(&Default::default(), "db2").unwrap();
        let cf3 = db.create_column_family(&Default::default(), "db3").unwrap();
        let cf4 = db.create_column_family(&Default::default(), "db4").unwrap();

        // via DB api
        assert!(
            db.put_cf(&WriteOptions::default(), &def, b"AA", b"aa")
                .is_ok()
        );
        assert!(
            db.put_cf(&WriteOptions::default(), &cf1, b"BB", b"bb")
                .is_ok()
        );
        assert!(
            db.put_cf(&WriteOptions::default(), &cf2, b"CC", b"cc")
                .is_ok()
        );
        assert!(
            db.put_cf(&WriteOptions::default(), &cf3, b"DD", b"dd")
                .is_ok()
        );
        assert!(
            db.put_cf(&WriteOptions::default(), &cf4, b"EE", b"ee")
                .is_ok()
        );

        // via CF api
        assert!(def.put(&WriteOptions::default(), b"AA", b"aa").is_ok());
        assert!(cf1.put(&WriteOptions::default(), b"BB", b"bb").is_ok());
        assert!(cf2.put(&WriteOptions::default(), b"CC", b"cc").is_ok());
        assert!(cf3.put(&WriteOptions::default(), b"DD", b"dd").is_ok());
        assert!(cf4.put(&WriteOptions::default(), b"EE", b"ee").is_ok());

        assert!(def.compact_range(&Default::default(), ..).is_ok());

        assert!(db.compact_range(&Default::default(), ..).is_ok());

        let ret = db.multi_get_cf(
            &ReadOptions::default(),
            &[&def, &cf1, &cf2, &cf3, &cf4, &def],
            &[b"AA", b"BB", b"CC", b"DD", b"EE", b"233"],
        );

        assert_eq!(ret[0].as_ref().unwrap(), b"aa".as_ref());
        assert_eq!(ret[2].as_ref().unwrap(), b"cc".as_ref());
        assert_eq!(ret[4].as_ref().unwrap(), b"ee".as_ref());
        assert!(ret[5].as_ref().unwrap_err().is_not_found());

        // mem::forget(def);
    }

    #[test]
    fn db_paths() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let dir1 = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let dir2 = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let wal_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();

        let opt = Options::default().map_db_options(|dbopt| {
            dbopt
                .create_if_missing(true)
                .db_paths(vec![&dir1.path(), &dir2.path()])        /* only puts sst file */
                .wal_dir(&wal_dir)
        });

        let db = DB::open(opt, &tmp_dir);
        if db.is_err() {
            println!("err => {:?}", db.unwrap_err());
            return;
        }
        let db = db.unwrap();
        let _ = db.put(&WriteOptions::default(), b"name", b"BH1XUW")
            .unwrap();
        for i in 0..10 {
            let key = format!("k{}", i);
            let value = format!("v{:03}", i);
            if i == 5 {
                let s = db.get_snapshot();
                assert!(s.is_some());
                db.release_snapshot(s.unwrap());
            }

            db.put(&WriteOptions::default(), key.as_bytes(), value.as_bytes())
                .unwrap();

            assert!(db.flush(&FlushOptions::default()).is_ok());
        }

        let d1 = dir1.path().read_dir().expect("should have a sst dir");
        let d2 = dir2.path().read_dir().expect("should have a sst dir");
        assert!(d1.count() > 0 || d2.count() > 0);

        let w = wal_dir.path().read_dir().expect("should have a wal dir");
        assert!(w.count() >= 1); // at least 1 log file

        let d = tmp_dir.path().read_dir().expect("should have a data dir");
        assert!(d.count() >= 2); // OPTIONS, MANIFEST, etc.
    }

    #[test]
    fn key_may_exist() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(Options::default().map_db_options(|db| db.create_if_missing(true)), &tmp_dir).unwrap();

        assert!(
            db.put(&Default::default(), b"long-key", b"long-value")
                .is_ok()
        );
        assert!(db.compact_range(&Default::default(), ..).is_ok());

        assert!(db.key_may_exist(&ReadOptions::default(), b"long-key"));
        assert!(!db.key_may_exist(&ReadOptions::default(), b"long-key-not-exist"));

        let (found, maybe_val) = db.key_may_get(&ReadOptions::default(), b"long-key");
        assert!(found);
        // it depends, Some/None are all OK
        // assert!(maybe_val.is_some());

        let (found, maybe_val) = db.key_may_get(&ReadOptions::default(), b"not-exist");
        assert!(!found);
        assert!(!maybe_val.is_some());
    }

    #[test]
    fn get_prop() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(Options::default().map_db_options(|db| db.create_if_missing(true)), &tmp_dir).unwrap();

        assert!(
            db.put(&Default::default(), b"long-key", vec![b'A'; 1024 * 1024].as_ref())
                .is_ok()
        );

        let cf1 = db.create_column_family(&Default::default(), "db1").unwrap();

        assert!(db.compact_range(&Default::default(), ..).is_ok());

        let snap = db.get_snapshot();
        assert_eq!(db.get_property("rocksdb.num-snapshots"), Some("1".to_string()));

        // dump status
        println!("stats => {}", db.get_property("rocksdb.stats").unwrap());
        assert_eq!(db.get_int_property("rocksdb.num-snapshots"), Some(1));

        assert!(
            db.put(&Default::default(), b"long-key2", vec![b'A'; 1024 * 1024].as_ref())
                .is_ok()
        );

        assert!(
            cf1.put(&Default::default(), b"long-key2", vec![b'A'; 1024 * 1024].as_ref())
                .is_ok()
        );

        assert!(db.get_int_property("rocksdb.size-all-mem-tables").unwrap() < 2 * 1024 * 1024);

        assert!(
            db.get_aggregated_int_property("rocksdb.size-all-mem-tables")
                .unwrap() > 2 * 1024 * 1024
        );

        db.release_snapshot(snap.unwrap());

    }

    #[test]
    fn misc_functions() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(
            Options::default()
                .map_db_options(|db| db.create_if_missing(true))
                .map_cf_options(|cf| cf.disable_auto_compactions(true)),
            &tmp_dir,
        ).unwrap();

        assert!(
            db.put(&Default::default(), b"long-key", vec![b'A'; 1024 * 1024].as_ref())
                .is_ok()
        );
        assert!(db.put(&Default::default(), b"a", b"1").is_ok());
        assert!(db.put(&Default::default(), b"b", b"2").is_ok());
        assert!(db.put(&Default::default(), b"c", b"3").is_ok());

        assert!(db.compact_range(&Default::default(), ..).is_ok());

        assert!(db.pause_background_work().is_ok());
        assert!(db.continue_background_work().is_ok());

        assert!(
            db.enable_auto_compaction(&[&db.default_column_family()])
                .is_ok()
        );

        assert_eq!(db.number_levels(), 7); // default
        assert_eq!(db.max_mem_compaction_level(), 0); // TODO: wtf
        assert_eq!(db.level0_stop_write_trigger(), 36); // default

        assert!(db.get_db_identity().is_ok());
        println!("id => {:?}", db.get_db_identity());
    }

    #[test]
    fn flush() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(
            Options::default()
                .map_db_options(|db| db.create_if_missing(true))
                .map_cf_options(|cf| cf.disable_auto_compactions(true)),
            &tmp_dir,
        ).unwrap();

        assert_eq!(*db.get_latest_sequence_number(), 0);

        assert!(
            db.put(&Default::default(), b"long-key", vec![b'A'; 1024 * 1024].as_ref())
                .is_ok()
        );
        assert!(db.put(&Default::default(), b"a", b"1").is_ok());
        assert!(db.put(&Default::default(), b"b", b"2").is_ok());
        assert!(db.put(&Default::default(), b"c", b"3").is_ok());

        assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
        assert!(db.sync_wal().is_ok());

        // 5th transaction
        assert_eq!(*db.get_latest_sequence_number(), 4);
    }

    #[test]
    fn livemetadata() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(Options::default().map_db_options(|db| db.create_if_missing(true)), &tmp_dir).unwrap();

        assert!(db.disable_file_deletions().is_ok());
        let meta = db.get_live_files_metadata();
        assert_eq!(meta.len(), 0);

        assert!(
            db.put(&Default::default(), b"long-key", vec![b'A'; 1024 * 1024].as_ref())
                .is_ok()
        );
        assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
        let meta = db.get_live_files_metadata();
        assert_eq!(meta.len(), 1);
        assert_eq!(meta[0].level, 0);

        assert!(db.put(&Default::default(), b"a", b"1").is_ok());
        assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
        assert!(db.put(&Default::default(), b"b", b"2").is_ok());
        assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
        assert!(db.put(&Default::default(), b"c", b"3").is_ok());
        assert!(db.put(&Default::default(), b"d", b"3").is_ok());
        assert!(
            db.put(&Default::default(), b"asdlfkjasl", b"askdfjkl3")
                .is_ok()
        );
        assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
        let meta = db.get_live_files_metadata();
        assert_eq!(meta.len(), 4);
        assert!(
            db.compact_range(&CompactRangeOptions::default(), ..)
                .is_ok()
        );

        let meta = db.get_live_files_metadata();
        assert!(meta.len() < 4);
        assert_eq!(meta[0].level, 1);

    }

    #[test]
    fn column_family_meta() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(Options::default().map_db_options(|db| db.create_if_missing(true)), &tmp_dir).unwrap();

        for i in 0..10 {
            let key = format!("k{}", i);
            let val = format!("v{}", i * 10);

            db.put(&WriteOptions::default(), key.as_bytes(), val.as_bytes())
                .unwrap();

            // 2 keys into a sst
            if i % 2 == 0 {
                assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
            }

            // leave 6-9 uncompacted
            if i == 5 {
                assert!(
                    db.compact_range(&CompactRangeOptions::default(), ..)
                        .is_ok()
                );
            }
        }

        let meta = db.get_column_family_metadata(&db.default_column_family());
        println!("Meta => {:?}", meta);
        assert_eq!(meta.levels.len(), 7, "default level num");
        assert!(meta.levels[0].files.len() + meta.levels[1].files.len() > 1);
        assert!(meta.levels[4].files.len() == 0);
    }


    #[test]
    fn list_live_files() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(Options::default().map_db_options(|db| db.create_if_missing(true)), &tmp_dir).unwrap();
        assert!(
            db.put(&Default::default(), b"long-key", vec![b'A'; 1024 * 1024].as_ref())
                .is_ok()
        );
        assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
        assert!(
            db.put(&Default::default(), b"long-key-2", vec![b'A'; 2 * 1024].as_ref())
                .is_ok()
        );
        assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());

        if let Ok((size, files)) = db.get_live_files(false) {
            assert!(files.contains(&"/CURRENT".to_string()));
        } else {
            assert!(false, "get_live_files fails");
        }
    }

    #[test]
    fn get_sorted_wal_files() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(
            Options::default().map_db_options(|db| {
                db.create_if_missing(true)
                                  .db_write_buffer_size(2 << 20) // 2MB per wal log
                                  .wal_ttl_seconds(1000)
            }),
            &tmp_dir,
        ).unwrap();
        for i in 0..10 {
            assert!(
                db.put(&Default::default(),
                           format!("key{}", i).as_bytes(),
                           format!("val{:01000000}", i).as_bytes()) // 1MB value
                    .is_ok()
            );
        }
        let files = db.get_sorted_wal_files();
        assert!(files.is_ok());
        assert!(files.unwrap().len() > 2);
    }

    #[test]
    fn change_options() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(
            Options::default()
                .map_db_options(|db| db.create_if_missing(true))
                .map_cf_options(|cf| cf.disable_auto_compactions(true)), // disable
            &tmp_dir,
        ).unwrap();
        assert!(
            db.put(&Default::default(), b"long-key", vec![b'A'; 1024 * 1024].as_ref())
                .is_ok()
        );
        assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
        assert!(
            db.put(&Default::default(), b"long-key-2", vec![b'A'; 2 * 1024].as_ref())
                .is_ok()
        );


        let new_opt: HashMap<&str, &str> = [("base_background_compactions", "6"),
             ("stats_dump_period_sec", "10")] // dump every 10s
            .iter().cloned().collect();
        let ret = db.set_db_options(&new_opt);
        assert!(ret.is_ok());

        let new_opt: HashMap<&str, &str> = [
            ("write_buffer_size", "10000000"),
            ("level0_file_num_compaction_trigger", "2"),
        ].iter()
            .cloned()
            .collect();
        assert!(db.set_options(&new_opt).is_ok());


        let new_opt: HashMap<&str, &str> = [("non-exist-write_buffer_size", "10000000")]
            .iter()
            .cloned()
            .collect();
        let ret = db.set_options(&new_opt);
        assert!(ret.is_err());
        assert!(format!("{:?}", ret).contains("Unrecognized option"));

    }

    #[test]
    fn approximate_sizes() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(
            Options::default()
                .map_db_options(|db| db.create_if_missing(true))
                .map_cf_options(|cf| cf.disable_auto_compactions(true)), // disable
            &tmp_dir,
        ).unwrap();
        assert!(
            db.put(&Default::default(), b"long-key", vec![b'A'; 1024 * 1024].as_ref())
                .is_ok()
        );
        assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
        assert!(
            db.put(&Default::default(), b"long-key-2", vec![b'A'; 2 * 1024].as_ref())
                .is_ok()
        );

        let sizes = db.get_approximate_sizes(&[b"long-key".as_ref()..&b"long-key-".as_ref()]);
        assert_eq!(sizes.len(), 1);
        assert!(sizes[0] > 0);

        for i in 0..100 {
            let key = format!("k{}", i);
            let val = format!("v{}", i * 10);

            db.put(&WriteOptions::default(), key.as_bytes(), val.as_bytes())
                .unwrap();
        }
        let (count, size) = db.get_approximate_memtable_stats(b"a".as_ref()..&b"z".as_ref());
        assert!(count > 0 && count < 200);
        assert!(size > 0);
    }


    #[test]
    fn compact_files() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(
            Options::default()
                .map_db_options(|db| db.create_if_missing(true))
                .map_cf_options(|cf| cf.disable_auto_compactions(true)), // disable
            &tmp_dir,
        ).unwrap();
        assert!(
            db.put(&Default::default(), b"long-key", vec![b'A'; 1024 * 1024].as_ref())
                .is_ok()
        );
        assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
        assert!(
            db.put(&Default::default(), b"long-key-2", vec![b'A'; 2 * 1024].as_ref())
                .is_ok()
        );

        for i in 0..10 {
            let key = format!("k{}", i);
            let val = format!("v{}", i * 10);

            db.put(&WriteOptions::default(), key.as_bytes(), val.as_bytes())
                .unwrap();

            if i % 2 == 0 {
                assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
            }
        }
        let v = db.get_live_files(true);

        let sst_files = v.as_ref()
            .unwrap()
            .1
            .iter()
            .filter(|name| name.ends_with(".sst"))
            .map(|name| name.as_ref())
            .collect::<Vec<&str>>();
        assert!(sst_files.len() > 2); // many sst files

        let st = db.compact_files(
            &CompactionOptions::default().compression(CompressionType::BZip2Compression),
            &sst_files,
            4,
        ); // output to level 4

        let result = db.get_live_files_metadata();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].level, 4); // compacted to 4
    }

    #[test]
    fn get_properties_of_all_tables() {
        let tmp_dir = ::tempdir::TempDir::new_in("", "rocks").unwrap();
        let db = DB::open(
            Options::default()
                .map_db_options(|db| db.create_if_missing(true))
                .map_cf_options(|cf| cf.disable_auto_compactions(true)),
            &tmp_dir,
        ).unwrap();

        for i in 0..10 {
            let key = format!("k{}", i);
            let val = format!("v{}", i * i);

            db.put(WriteOptions::default_instance(), key.as_bytes(), val.as_bytes())
                .unwrap();

            if i % 2 == 0 {
                assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
            }
        }

        let props = db.get_properties_of_all_tables_cf(&db.default_column_family());
        assert!(props.is_ok());
        let props = props.unwrap();
        assert!(props.len() > 4, "should be more than 4 sst files");

        for (k, prop) in props.iter() {
            println!("key => {:?}", k);
            println!("    => {:?}", prop);
            println!("data size ={}", prop.data_size());
            assert_eq!(prop.column_family_name(), Some("default"));

            println!("filter policy name = {:?}", prop.filter_policy_name());
            println!("comparator name = {:?}", prop.comparator_name());

            assert_eq!(prop.property_collectors_names(), "[]");

            let user_prop = prop.user_collected_properties();
            println!("len => {:?}", user_prop.len());
            for (k, v) in user_prop.iter() {
                println!("    {}=>{:?}", k, v);
            }
            let readable_prop = prop.readable_properties();
            println!("readable => {:?}", readable_prop);
        }

        let vals = props.iter().map(|(k, _)| k).collect::<Vec<_>>();
        assert!(vals.len() > 4);
    }

    #[test]
    fn delete_files_in_range() {
        let tmp_dir = ::tempdir::TempDir::new_in("", "rocks").unwrap();
        let db = DB::open(
            Options::default().map_db_options(|db| db.create_if_missing(true)),
            // NOTE: disable auto compaction
            // .map_cf_options(|cf| cf.disable_auto_compactions(true)),
            &tmp_dir,
        ).unwrap();

        // will have 10 sst file
        for i in 0..10 {
            let key = format!("k{}", i);
            let val = format!("v{}", i * i);

            db.put(WriteOptions::default_instance(), key.as_bytes(), val.as_bytes())
                .unwrap();

            assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
        }

        // NOTE: size is manifest_file_size, not total size
        let (_old_size, old_files) = db.get_live_files(false).expect("should get live files");

        assert!(
            db.delete_files_in_range(&db.default_column_family(), b"k2", b"k8")
                .is_ok()
        );

        let (_new_size, new_files) = db.get_live_files(false).expect("should get live files");

        assert!(new_files.len() < old_files.len());
        for f in &new_files {
            assert!(old_files.contains(f));
        }
    }
}
