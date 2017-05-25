//! A DB is a persistent ordered map from keys to values.

use std::mem;
use std::ffi::{CStr, CString};
use std::os::raw::{c_int, c_char, c_void};
use std::ptr;
use std::iter;
use std::str;
use std::slice;
use std::rc::{Rc, Weak};
use std::ops;
use std::fmt;
use std::iter::IntoIterator;
use std::marker::PhantomData;
use std::path::Path;

use rocks_sys as ll;

use status::Status;
use comparator::Comparator;
use options::{Options, DBOptions, ColumnFamilyOptions, ReadOptions, WriteOptions,
              CompactRangeOptions, IngestExternalFileOptions, FlushOptions};
use table_properties::TableProperties;
use snapshot::Snapshot;
use write_batch::WriteBatch;
use iterator::Iterator;
use merge_operator::{MergeOperator, AssociativeMergeOperator};
use env::Logger;
use types::SequenceNumber;

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

pub struct ColumnFamilyHandle<'a, 'b: 'a> {
    raw: *mut ll::rocks_column_family_handle_t,
    db: Rc<DBContext<'b>>, // 'b out lives 'a
    owned: bool,
    _marker: PhantomData<&'a ()>,
}

impl<'a, 'b> Drop for ColumnFamilyHandle<'a, 'b> {
    fn drop(&mut self) {
        // unsafe { ll::rocks_column_family_handle_destroy(self.raw) }
        unsafe {
            if self.owned {
                let mut status = mem::zeroed();
                ll::rocks_db_destroy_column_family_handle(self.db.raw, self.raw(), &mut status);
                assert!(status.code == 0);
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
        write!(f, "CFHandle({:?})", self.raw)
    }
}

impl<'a, 'b: 'a> ColumnFamilyHandle<'a, 'b> {
    pub fn raw(&self) -> *mut ll::rocks_column_family_handle_t {
        self.raw
    }

    // FIXME: should be unsafe
    // fn from_ll(raw: *mut ll::rocks_column_family_handle_t, _: *mut ll::rocks_db_t) -> ColumnFamilyHandle<'a> {
    //      ColumnFamilyHandle {
    //          raw: raw,
    //          db:
    //      }
    //  }

    /// Returns the name of the column family associated with the current handle.
    pub fn get_name(&self) -> &str {
        unsafe {
            let ptr = ll::rocks_column_family_handle_get_name(self.raw);
            CStr::from_ptr(ptr).to_str().unwrap()
        }
    }

    /// Returns the ID of the column family associated with the current handle.
    pub fn get_id(&self) -> u32 {
        unsafe { ll::rocks_column_family_handle_get_id(self.raw) }
    }

    /// Fills "*desc" with the up-to-date descriptor of the column family
    /// associated with this handle. Since it fills "*desc" with the up-to-date
    /// information, this call might internally lock and release DB mutex to
    /// access the up-to-date CF options.  In addition, all the pointer-typed
    /// options cannot be referenced any longer than the original options exist.
    ///
    /// Note that this function is not supported in RocksDBLite.
    pub fn get_descriptor(&self,
                          desc: &mut ColumnFamilyDescriptor)
                          -> Result<ColumnFamilyDescriptor, Status> {
        unimplemented!()
    }

    /*
    /// Returns the comparator of the column family associated with the
    /// current handle.
    //pub fn get_comparator(&self) -> Comparator {
    //unimplemented!()
    //}
     */
}

struct DBContext<'a> {
    raw: *mut ll::rocks_db_t,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Drop for DBContext<'a> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            ll::rocks_db_close(self.raw);
        }
    }
}


/// A `DB` is a persistent ordered map from keys to values.
///
/// A `DB` is safe for concurrent access from multiple threads without
/// any external synchronization.
///
/// # Examples
///
/// ```
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
    context: Rc<DBContext<'a>>,
}

impl<'a> fmt::Debug for DB<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DB({:?})", self.context.raw)
    }
}

unsafe impl<'a> Sync for DB<'a> {}

impl<'a> DB<'a> {
    pub unsafe fn from_ll<'b>(raw: *mut ll::rocks_db_t) -> DB<'b> {
        let context = DBContext {
            raw: raw,
            _marker: PhantomData,
        };
        DB { context: Rc::new(context) }
    }

    pub fn raw(&self) -> *mut ll::rocks_db_t {
        self.context.raw
    }

    /// Open the database with the specified `name`.
    ///
    /// Stores a pointer to a heap-allocated database in *dbptr and returns
    /// OK on success.
    ///
    /// Stores nullptr in *dbptr and returns a non-OK status on error.
    /// Caller should delete *dbptr when it is no longer needed.
    pub fn open<'b, T: AsRef<Options>, P: AsRef<Path>>(options: T,
                                                       name: P)
                                                       -> Result<DB<'b>, Status> {
        unsafe {
            let opt = options.as_ref().raw();
            let dbname = name.as_ref()
                .to_str()
                .and_then(|s| CString::new(s).ok())
                .unwrap();
            let mut status = mem::zeroed::<ll::rocks_status_t>();
            let db_ptr = ll::rocks_db_open(opt, dbname.as_ptr(), &mut status);
            if status.code == 0 {
                Ok(DB::from_ll(db_ptr))
            } else {
                Err(Status::from_ll(&status))
            }
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
    /// Before delete DB, you have to close All column families by calling
    /// `DestroyColumnFamilyHandle()` with all the handles.
    pub fn open_with_column_families<'b, 'c: 'b, CF: Into<ColumnFamilyDescriptor>>
        (options: &Options,     // FIXME: this should be DBOptions
         name: &str,
         column_families: Vec<CF>)
         -> Result<(DB<'b>, Vec<ColumnFamilyHandle<'c, 'b>>), Status> {
        unsafe {
            let opt = options.raw();
            let mut status = mem::uninitialized::<ll::rocks_status_t>();
            let dbname = CString::new(name).unwrap();

            let cfs: Vec<ColumnFamilyDescriptor> = column_families
                .into_iter()
                .map(|desc| desc.into())
                .collect();

            let num_column_families = cfs.len();
            // for ffi
            let mut cfnames: Vec<*const c_char> = Vec::with_capacity(num_column_families);
            let mut cfopts: Vec<*const ll::rocks_cfoptions_t> =
                Vec::with_capacity(num_column_families);
            let mut cfhandles = vec![ptr::null_mut(); num_column_families];

            // FIXME: is it necessary to create?
            // hold it to call ffi function
            let default_cfopt = ColumnFamilyOptions::from_options(options);

            for cf in &cfs {
                cfnames.push(cf.name_as_ptr());
                cfopts.push(cf.options.as_ref().unwrap_or_else(|| &default_cfopt).raw());
            }

            let db_ptr = ll::rocks_db_open_column_families(options.raw(),
                                                           dbname.as_ptr(),
                                                           num_column_families as c_int,
                                                           cfnames.as_ptr(),
                                                           cfopts.as_ptr(),
                                                           cfhandles.as_mut_ptr(),
                                                           &mut status);

            if status.code == 0 {
                let db = DB::from_ll(db_ptr);
                let db_ctx = db.context.clone();
                Ok((db,
                    cfhandles
                        .into_iter()
                        .map(move |p| {
                                 ColumnFamilyHandle {
                                     raw: p,
                                     db: db_ctx.clone(),
                                     owned: true,
                                     _marker: PhantomData,
                                 }
                             })
                        .collect()))
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }

    /// Open the database for read only. All DB interfaces
    /// that modify data, like `put/delete`, will return error.
    /// If the db is opened in read only mode, then no compactions
    /// will happen.
    ///
    /// Not supported in ROCKSDB_LITE, in which case the function will
    /// return `Status::NotSupported`.
    pub fn open_for_readonly<'b>(options: &Options,
                                 name: &str,
                                 error_if_log_file_exist: bool)
                                 -> Result<DB<'b>, Status> {
        unsafe {
            let dbname = CString::new(name).unwrap();
            let mut status = mem::zeroed::<ll::rocks_status_t>();
            let db_ptr = ll::rocks_db_open_for_read_only(options.raw(),
                                                         dbname.as_ptr(),
                                                         error_if_log_file_exist as u8,
                                                         &mut status);
            if status.code == 0 {
                Ok(DB::from_ll(db_ptr))
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }


    /// `ListColumnFamilies` will open the DB specified by argument name
    /// and return the list of all column families in that DB
    /// through `column_families` argument. The ordering of
    /// column families in column_families is unspecified.
    pub fn list_column_families(options: &Options, name: &str) -> Result<Vec<String>, Status> {
        unsafe {
            let dbname = CString::new(name).unwrap();
            let mut status = mem::zeroed::<ll::rocks_status_t>();
            let mut lencfs = 0;
            let cfs = ll::rocks_db_list_column_families(options.raw(),
                                                        dbname.as_ptr(),
                                                        &mut lencfs,
                                                        &mut status);
            if status.code == 0 {
                if lencfs == 0 {
                    Ok(vec![])
                } else {
                    let mut ret = Vec::with_capacity(lencfs);
                    for i in 0..lencfs {
                        ret.push(CStr::from_ptr(*cfs.offset(i as isize))
                                     .to_str()
                                     .unwrap()
                                     .to_string());
                    }
                    ll::rocks_db_list_column_families_destroy(cfs, lencfs);
                    Ok(ret)
                }
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }

    /// Create a column_family and return the handle of column family
    /// through the argument handle.
    pub fn create_column_family(&self,
                                cfopts: &ColumnFamilyOptions,
                                column_family_name: &str)
                                -> Result<ColumnFamilyHandle, Status> {
        unsafe {
            let dbname = CString::new(column_family_name).unwrap();
            let mut status = mem::uninitialized::<ll::rocks_status_t>();

            let handle = ll::rocks_db_create_column_family(self.raw(),
                                                           cfopts.raw(),
                                                           dbname.as_ptr(),
                                                           &mut status);
            if status.code == 0 {
                Ok(ColumnFamilyHandle {
                       raw: handle,
                       db: self.context.clone(),
                       owned: true,
                       _marker: PhantomData,
                   })
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }

    pub fn put_cf(&self,
                  options: &WriteOptions,
                  column_family: &ColumnFamilyHandle,
                  key: &[u8],
                  value: &[u8])
                  -> Result<(), Status> {
        unsafe {
            let mut status = mem::zeroed::<ll::rocks_status_t>();
            // since rocksdb::DB::put without cf is for compatibility
            ll::rocks_db_put_cf(self.raw(),
                                options.raw(),
                                column_family.raw(),
                                key.as_ptr() as _,
                                key.len(),
                                value.as_ptr() as _,
                                value.len(),
                                &mut status);
            if status.code == 0 {
                Ok(())
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }

    /// Set the database entry for `"key"` to `"value"`.
    /// If `"key"` already exists, it will be overwritten.
    /// Returns OK on success, and a non-OK status on error.
    ///
    /// Note: consider setting `options.sync = true`.
    pub fn put(&self, options: &WriteOptions, key: &[u8], value: &[u8]) -> Result<(), Status> {
        unsafe {
            let mut status = mem::zeroed::<ll::rocks_status_t>();
            ll::rocks_db_put(self.raw(),
                             options.raw(),
                             key.as_ptr() as _,
                             key.len(),
                             value.as_ptr() as _,
                             value.len(),
                             &mut status);
            if status.code == 0 {
                Ok(())
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }

    // pub fn put_slice(&self, options: &WriteOptions,
    // key: &[u8], value: &[u8]) -> Result<(), Status> {
    // unsafe {
    // let mut status = mem::uninitialized::<ll::rocks_status_t>();
    // ll::rocks_db_put_slice(
    // self.raw,
    // options.raw(),
    // mem::transmute::<&&[u8], *const ll::Slice>(&key),
    // mem::transmute::<&&[u8], *const ll::Slice>(&value),
    // &mut status);
    // if status.code == 0 {
    // Ok(())
    // } else {
    // Err(Status::from_ll(&status))
    // }
    // }
    // }
    //

    /// Remove the database entry (if any) for "key".  Returns OK on
    /// success, and a non-OK status on error.  It is not an error if "key"
    /// did not exist in the database.
    ///
    /// Note: consider setting `options.sync = true`.
    pub fn delete<W: AsRef<WriteOptions>>(&self, options: W, key: &[u8]) -> Result<(), Status> {
        unsafe {
            let mut status = mem::zeroed();
            ll::rocks_db_delete(self.raw(),
                                options.as_ref().raw(),
                                key.as_ptr() as *const _,
                                key.len(),
                                &mut status);
            if status.code == 0 {
                Ok(())
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }

    pub fn delete_cf<W: AsRef<WriteOptions>>(&self,
                                             options: W,
                                             column_family: &ColumnFamilyHandle,
                                             key: &[u8])
                                             -> Result<(), Status> {
        unsafe {
            let mut status = mem::zeroed();
            ll::rocks_db_delete_cf(self.raw(),
                                   options.as_ref().raw(),
                                   column_family.raw(),
                                   key.as_ptr() as *const _,
                                   key.len(),
                                   &mut status);
            if status.code == 0 {
                Ok(())
            } else {
                Err(Status::from_ll(&status))
            }
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
    pub fn single_delete<W: AsRef<WriteOptions>>(&self,
                                                 options: W,
                                                 key: &[u8])
                                                 -> Result<(), Status> {
        unsafe {
            let mut status = mem::zeroed();
            ll::rocks_db_single_delete(self.raw(),
                                       options.as_ref().raw(),
                                       key.as_ptr() as *const _,
                                       key.len(),
                                       &mut status);
            if status.code == 0 {
                Ok(())
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }

    pub fn single_delete_cf<W: AsRef<WriteOptions>>(&self,
                                                    options: W,
                                                    column_family: &ColumnFamilyHandle,
                                                    key: &[u8])
                                                    -> Result<(), Status> {
        unsafe {
            let mut status = mem::zeroed();
            ll::rocks_db_single_delete_cf(self.raw(),
                                          options.as_ref().raw(),
                                          column_family.raw(),
                                          key.as_ptr() as *const _,
                                          key.len(),
                                          &mut status);
            if status.code == 0 {
                Ok(())
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }

    /// Removes the database entries in the range ["begin_key", "end_key"), i.e.,
    /// including "begin_key" and excluding "end_key". Returns OK on success, and
    /// a non-OK status on error. It is not an error if no keys exist in the range
    /// ["begin_key", "end_key").
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
    pub fn delete_range_cf<W: AsRef<WriteOptions>>(&self,
                                                   options: W,
                                                   column_family: &ColumnFamilyHandle,
                                                   begin_key: &[u8],
                                                   end_key: &[u8])
                                                   -> Result<(), Status> {
        unsafe {
            let mut status = mem::zeroed();
            ll::rocks_db_delete_range_cf(self.raw(),
                                         options.as_ref().raw(),
                                         column_family.raw(),
                                         begin_key.as_ptr() as *const _,
                                         begin_key.len(),
                                         begin_key.as_ptr() as *const _,
                                         begin_key.len(),
                                         &mut status);
            if status.code == 0 {
                Ok(())
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }

    /// Merge the database entry for "key" with "value".  Returns OK on success,
    /// and a non-OK status on error. The semantics of this operation is
    /// determined by the user provided merge_operator when opening DB.
    ///
    /// Note: consider setting `options.sync = true`.
    pub fn merge<W: AsRef<WriteOptions>>(&self,
                                         options: W,
                                         key: &[u8],
                                         val: &[u8])
                                         -> Result<(), Status> {
        unsafe {
            let mut status = mem::zeroed();
            ll::rocks_db_merge(self.raw(),
                               options.as_ref().raw(),
                               key.as_ptr() as *const _,
                               key.len(),
                               val.as_ptr() as *const _,
                               val.len(),
                               &mut status);
            if status.code == 0 {
                Ok(())
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }

    pub fn merge_cf<W: AsRef<WriteOptions>>(&self,
                                            options: W,
                                            column_family: &ColumnFamilyHandle,
                                            key: &[u8],
                                            val: &[u8])
                                            -> Result<(), Status> {
        unsafe {
            let mut status = mem::zeroed();
            ll::rocks_db_merge_cf(self.raw(),
                                  options.as_ref().raw(),
                                  column_family.raw(),
                                  key.as_ptr() as *const _,
                                  key.len(),
                                  val.as_ptr() as *const _,
                                  val.len(),
                                  &mut status);
            if status.code == 0 {
                Ok(())
            } else {
                Err(Status::from_ll(&status))
            }
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
    pub fn write<W: AsRef<WriteOptions>>(&self,
                                         options: W,
                                         updates: WriteBatch)
                                         -> Result<(), Status> {
        unsafe {
            let mut status = mem::zeroed();
            ll::rocks_db_write(self.raw(),
                               options.as_ref().raw(),
                               updates.raw(),
                               &mut status);
            if status.code == 0 {
                Ok(())
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }

    /// If the database contains an entry for "key" store the
    /// corresponding value in *value and return OK.
    ///
    /// If there is no entry for "key" leave *value unchanged and return
    /// a status for which Status::IsNotFound() returns true.
    ///
    /// May return some other Status on an error.
    pub fn get<R: AsRef<ReadOptions>>(&self, options: R, key: &[u8]) -> Result<CVec<u8>, Status> {
        unsafe {
            let mut status = mem::zeroed::<ll::rocks_status_t>();
            let mut vallen = 0_usize;
            let ptr = ll::rocks_db_get(self.raw(),
                                       options.as_ref().raw(),
                                       key.as_ptr() as _,
                                       key.len(),
                                       &mut vallen,
                                       &mut status);

            if status.code == 0 {
                Ok(CVec::from_raw_parts(ptr as *mut u8, vallen))
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }

    pub fn get_cf(&self,
                  options: &ReadOptions,
                  column_family: &ColumnFamilyHandle,
                  key: &[u8])
                  -> Result<CVec<u8>, Status> {
        unsafe {
            let mut status = mem::zeroed::<ll::rocks_status_t>();
            let mut vallen = 0_usize;
            let ptr = ll::rocks_db_get_cf(self.raw(),
                                          options.raw(),
                                          column_family.raw(),
                                          key.as_ptr() as _,
                                          key.len(),
                                          &mut vallen,
                                          &mut status);

            if status.code == 0 {
                Ok(CVec::from_raw_parts(ptr as *mut u8, vallen))
            } else {
                Err(Status::from_ll(&status))
            }
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
    pub fn multi_get(&self,
                     options: &ReadOptions,
                     keys: &[&[u8]])
                     -> Vec<Result<CVec<u8>, Status>> {
        unsafe {
            let num_keys = keys.len();
            let mut c_keys: Vec<*const c_char> = Vec::with_capacity(num_keys);
            let mut c_keys_lens = Vec::with_capacity(num_keys);

            let mut vals = vec![ptr::null_mut(); num_keys];
            let mut vals_lens = vec![0_usize; num_keys];

            for i in 0..num_keys {
                c_keys.push(keys[i].as_ptr() as *const c_char);
                c_keys_lens.push(keys[i].len());
            }

            let mut status: Vec<ll::rocks_status_t> = vec![mem::zeroed(); num_keys];
            let mut ret = Vec::with_capacity(num_keys);

            ll::rocks_db_multi_get(self.raw(),
                                   options.raw(),
                                   num_keys,
                                   c_keys.as_ptr(),
                                   c_keys_lens.as_ptr(),
                                   vals.as_mut_ptr(),
                                   vals_lens.as_mut_ptr(),
                                   &mut status[0] as *mut _);

            for i in 0..num_keys {
                if status[i].code == 0 {
                    ret.push(Ok(CVec::from_raw_parts(vals[i] as *mut u8, vals_lens[i])));
                } else {
                    ret.push(Err(Status::from_ll(&status[i])))
                }
            }
            ret
        }
    }

    pub fn multi_get_cf(&self,
                        options: &ReadOptions,
                        column_families: &[&ColumnFamilyHandle],
                        keys: &[&[u8]])
                        -> Vec<Result<CVec<u8>, Status>> {
        unsafe {
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

            let mut status: Vec<ll::rocks_status_t> = vec![mem::zeroed(); num_keys];
            let mut ret = Vec::with_capacity(num_keys);

            ll::rocks_db_multi_get_cf(self.raw(),
                                      options.raw(),
                                      c_cfs.as_ptr(),
                                      num_keys,
                                      c_keys.as_ptr(),
                                      c_keys_lens.as_ptr(),
                                      vals.as_mut_ptr(),
                                      vals_lens.as_mut_ptr(),
                                      &mut status[0] as *mut _);

            for i in 0..num_keys {
                if status[i].code == 0 {
                    ret.push(Ok(CVec::from_raw_parts(vals[i] as *mut u8, vals_lens[i])));
                } else {
                    ret.push(Err(Status::from_ll(&status[i])))
                }
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
            ll::rocks_db_key_may_exist(self.raw(),
                                       options.raw(),
                                       key.as_ptr() as *const _,
                                       key.len(),
                                       ptr::null_mut(),
                                       ptr::null_mut(),
                                       ptr::null_mut()) != 0
        }
    }

    pub fn key_may_get(&self, options: &ReadOptions, key: &[u8]) -> (bool, Option<CVec<u8>>) {
        unsafe {
            let mut found = 0;
            let mut value: *mut c_char = ptr::null_mut();
            let mut value_len: usize = 0;
            let ret = ll::rocks_db_key_may_exist(self.raw(),
                                                 options.raw(),
                                                 key.as_ptr() as *const _,
                                                 key.len(),
                                                 &mut value,
                                                 &mut value_len,
                                                 &mut found);
            if ret == 0 {
                (false, None)
            } else if found == 0 {
                (true, None)
            } else {
                (true, Some(CVec::from_raw_parts(value as *mut _, value_len)))
            }
        }
    }

    pub fn key_may_exist_cf(&self,
                            options: &ReadOptions,
                            column_family: &ColumnFamilyHandle,
                            key: &[u8])
                            -> bool {
        unsafe {
            ll::rocks_db_key_may_exist_cf(self.raw(),
                                          options.raw(),
                                          column_family.raw(),
                                          key.as_ptr() as *const _,
                                          key.len(),
                                          ptr::null_mut(),
                                          ptr::null_mut(),
                                          ptr::null_mut()) != 0
        }
    }

    pub fn key_may_get_cf(&self,
                          options: &ReadOptions,
                          column_family: &ColumnFamilyHandle,
                          key: &[u8])
                          -> (bool, Option<CVec<u8>>) {
        unimplemented!()
    }

    /// Return a heap-allocated iterator over the contents of the database.
    /// The result of NewIterator() is initially invalid (caller must
    /// call one of the Seek methods on the iterator before using it).
    ///
    /// Caller should delete the iterator when it is no longer needed.
    /// The returned iterator should be deleted before this db is deleted.
    pub fn new_iterator(&self, options: &ReadOptions) -> Iterator {
        unsafe {
            let ptr = ll::rocks_db_create_iterator(self.raw(), options.raw());
            Iterator::from_ll(ptr)
        }
    }

    pub fn new_iterator_cf(&self, options: &ReadOptions, cf: &ColumnFamilyHandle) -> Iterator {
        unsafe {
            let ptr = ll::rocks_db_create_iterator_cf(self.raw(), options.raw(), cf.raw());
            Iterator::from_ll(ptr)
        }
    }

    pub fn new_iterators<'c, 'b: 'c, T: AsRef<ColumnFamilyHandle<'c, 'b>>>(&'b self,
                         options: &ReadOptions,
                         cfs: &[T])
                         -> Result<Vec<Iterator>, Status> {
        unsafe {
            let c_cfs = cfs.iter().map(|cf| cf.as_ref().raw()).collect::<Vec<_>>();
            let cfs_len = cfs.len();
            let mut status = mem::zeroed();

            let mut c_iters = vec![ptr::null_mut(); cfs_len];
            ll::rocks_db_create_iterators(self.raw(),
                                          options.raw(),
                                          c_cfs.as_ptr() as _,
                                          c_iters.as_mut_ptr(),
                                          cfs_len,
                                          &mut status);
            if status.code == 0 {
                Ok(c_iters
                       .into_iter()
                       .map(|ptr| Iterator::from_ll(ptr))
                       .collect())
            } else {
                Err(Status::from_ll(&status))
            }
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
        unsafe {
            let mut ret = String::new();
            if ll::rocks_db_get_property(self.raw(),
                                         property.as_bytes().as_ptr() as *const _,
                                         property.len(),
                                         &mut ret as *mut String as *mut c_void) != 0 {
                Some(ret)
            } else {
                None
            }
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
        unsafe {
            let mut val = 0;
            if ll::rocks_db_get_int_property(self.raw(),
                                             property.as_bytes().as_ptr() as *const _,
                                             property.len(),
                                             &mut val) != 0 {
                Some(val)
            } else {
                None
            }
        }
    }

    /// Same as GetIntProperty(), but this one returns the aggregated int
    /// property from all column families.
    pub fn get_aggregated_int_property(&self, property: &str) -> Option<u64> {
        unsafe {
            let mut val = 0;
            if ll::rocks_db_get_aggregated_int_property(self.raw(),
                                                        property.as_bytes().as_ptr() as *const _,
                                                        property.len(),
                                                        &mut val) != 0 {
                Some(val)
            } else {
                None
            }
        }
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
    pub fn get_approximate_sizes(&self, range: &[ops::Range<&[u8]>], include_flags: u8) -> u64 {
        unimplemented!()
    }

    /// The method is similar to GetApproximateSizes, except it
    /// returns approximate number of records in memtables.
    pub fn get_approximate_memtable_stats(&self, range: &[ops::Range<&[u8]>], include_flags: u8) -> u64 {
        unimplemented!()
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
    pub fn compact_range<R: ToCompactRange>(&self,
                                            options: &CompactRangeOptions,
                                            range: R)
                                            -> Result<(), Status> {
        unsafe {
            let mut status = mem::zeroed();
            ll::rocks_db_compact_range_opt(self.raw(),
                                           options.raw(),
                                           range.start_key() as *const _,
                                           range.start_key_len(),
                                           range.end_key() as *const _,
                                           range.end_key_len(),
                                           &mut status);
            if status.code == 0 {
                Ok(())
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }

    // set options
    // set dboptions via Map<K,V>
    // compact files


    /// This function will wait until all currently running background processes
    /// finish. After it returns, no background process will be run until
    /// UnblockBackgroundWork is called
    pub fn pause_background_work(&self) -> Result<(), Status> {
        unsafe {
            let mut status = mem::zeroed();
            ll::rocks_db_pause_background_work(self.raw(), &mut status);
            if status.code == 0 {
                Ok(())
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }

    pub fn continue_background_work(&self) -> Result<(), Status> {
        unsafe {
            let mut status = mem::zeroed();
            ll::rocks_db_continue_background_work(self.raw(), &mut status);
            if status.code == 0 {
                Ok(())
            } else {
                Err(Status::from_ll(&status))
            }
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
    pub fn enable_auto_compaction(&self,
                                  column_family_handles: &[&ColumnFamilyHandle])
                                  -> Result<(), Status> {
        unsafe {
            let c_cfs = column_family_handles.iter().map(|cf| cf.as_ref().raw() as *const _).collect::<Vec<*const _>>();
            let cfs_len = column_family_handles.len();
            let mut status = mem::zeroed();
            ll::rocks_db_enable_auto_compaction(self.raw(),
                                                c_cfs.as_ptr(),
                                                cfs_len,
                                                &mut status);
            if status.code == 0 {
                Ok(())
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }

    /// Number of levels used for this DB.
    pub fn number_levels(&self) -> u32 {
        unsafe {
            ll::rocks_db_number_levels(self.raw()) as u32
        }
    }

    /// Maximum level to which a new compacted memtable is pushed if it
    /// does not create overlap.
    pub fn max_mem_compaction_level(&self) -> u32 {
        unsafe {
            ll::rocks_db_max_mem_compaction_level(self.raw()) as u32
        }
    }

    /// Number of files in level-0 that would stop writes.
    pub fn level0_stop_write_trigger(&self) -> u32 {
        unsafe {
            ll::rocks_db_level0_stop_write_trigger(self.raw()) as u32
        }
    }

    /// Get DB name -- the exact same name that was provided as an argument to
    /// `DB::Open()`
    pub fn get_name(&self) -> &str {
        unsafe {
            let mut len = 0;
            let ptr = ll::rocks_db_get_name(self.raw(), &mut len);
            str::from_utf8_unchecked(slice::from_raw_parts(ptr as *const _, len))
        }
    }

    // get options

    // get db options

    /// Flush all mem-table data.
    pub fn flush(&self, options: &FlushOptions) -> Result<(), Status> {
        unsafe {
            let mut status = mem::zeroed();
            ll::rocks_db_flush(self.raw(), options.raw(), &mut status);
            if status.code == 0 {
                Ok(())
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }

    /// Sync the wal. Note that Write() followed by SyncWAL() is not exactly the
    /// same as Write() with sync=true: in the latter case the changes won't be
    /// visible until the sync is done.
    ///
    /// Currently only works if allow_mmap_writes = false in Options.
    pub fn sync_wal(&self) -> Result<(), Status> {
        unsafe {
            let mut status = mem::zeroed();
            ll::rocks_db_sync_wal(self.raw(), &mut status);
            if status.code == 0 {
                Ok(())
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }

    /// The sequence number of the most recent transaction.
    pub fn get_latest_sequence_number(&self) -> SequenceNumber {
        unsafe {
            ll::rocks_db_get_latest_sequence_number(self.raw())
        }
    }

    /// Prevent file deletions. Compactions will continue to occur,
    /// but no obsolete files will be deleted. Calling this multiple
    /// times have the same effect as calling it once.
    pub fn disable_file_deletions(&self) -> Result<(), Status> {
        unimplemented!()
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
    pub fn enable_file_deletions(&self, force: bool) -> Result<(), Status> {
        unimplemented!()
    }


    // TODO:
    // get_live_files
    // get_sorted_wal_files
    // get_updates_since
    // delete_file
    // get_live_files_metadata
    // get_column_family_metadata

    /// `IngestExternalFile()` will load a list of external SST files (1) into the DB
    /// We will try to find the lowest possible level that the file can fit in, and
    /// ingest the file into this level (2). A file that have a key range that
    /// overlap with the memtable key range will require us to Flush the memtable
    /// first before ingesting the file.
    ///
    /// - External SST files can be created using SstFileWriter
    /// - We will try to ingest the files to the lowest possible level
    ///   even if the file compression dont match the level compression
    pub fn ingest_external_file(&self,
                                external_files: &[String],
                                options: &IngestExternalFileOptions)
                                -> Result<(), Status> {
        unsafe {
            let mut status = mem::zeroed();
            let num_files = external_files.len();
            let mut c_files = Vec::with_capacity(num_files);
            let mut c_files_lens = Vec::with_capacity(num_files);
            for f in external_files {
                c_files.push(f.as_ptr() as *const _);
                c_files_lens.push(f.len());
            }
            ll::rocks_db_ingest_external_file(self.raw(),
                                              c_files.as_ptr() as *const _,
                                              c_files_lens.as_ptr(),
                                              num_files,
                                              options.raw(),
                                              &mut status);
            if status.code == 0 {
                Ok(())
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }

    /// Sets the globally unique ID created at database creation time by invoking
    /// `Env::GenerateUniqueId()`, in identity. Returns Status::OK if identity could
    /// be set properly
    pub fn get_db_identity(&self) -> Result<String, Status> {
        unsafe {
            let mut identity = String::new();
            let mut status = mem::zeroed();
            ll::rocks_db_get_db_identity(self.raw(),
                                         &mut identity as *mut String as *mut _,
                                         &mut status);
            if status.code == 0 {
                Ok(identity)
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }

    /// Returns default column family handle
    pub fn default_column_family(&self) -> ColumnFamilyHandle {
        ColumnFamilyHandle {
            raw: unsafe { ll::rocks_db_default_column_family(self.raw()) },
            db: self.context.clone(),
            owned: false,
            _marker: PhantomData,
        }
    }

    // TODO:
    // GetPropertiesOfAllTables
    // GetPropertiesOfTablesInRange
}


// public functions

/// Destroy the contents of the specified database.
///
/// Be very careful using this method.
pub fn destroy_db(name: &str, options: &Options) -> Result<(), Status> {
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
pub fn repair_db(dbname: &str, db_options: &DBOptions,
                 column_families: &[&ColumnFamilyDescriptor]) -> Result<(), Status> {
    unimplemented!()
}

/// `unknown_cf_opts` Options for column families encountered during the
/// repair that were not specified in `column_families`.
pub fn repair_db_with_unknown_cf_opts(dbname: &str, db_options: &DBOptions,
                                      column_families: &[&ColumnFamilyDescriptor],
                                      unknown_cf_opts: &ColumnFamilyOptions) -> Result<(), Status> {
    unimplemented!()
}

/// `options` These options will be used for the database and for ALL column
/// families encountered during the repair.
pub fn repair_db_all_cfs(dbname: &str, options: &Options) -> Result<(), Status> {
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

pub struct CVec<T> {
    data: *mut T,
    len: usize,
}

impl<T> CVec<T> {
    pub unsafe fn from_raw_parts(p: *mut T, len: usize) -> CVec<T> {
        CVec { data: p, len: len }
    }
}

impl CVec<u8> {
    pub fn to_str(&self) -> Result<&str, str::Utf8Error> {
        str::from_utf8(self)
    }
}

impl<T: fmt::Debug> fmt::Debug for CVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe { slice::from_raw_parts(self.data, self.len).fmt(f) }
    }
}

impl<T> ops::Deref for CVec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.data, self.len) }
    }
}

impl<T> AsRef<[T]> for CVec<T> {
    fn as_ref(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.data, self.len) }
    }
}

impl<T> Drop for CVec<T> {
    fn drop(&mut self) {
        unsafe {
            ll::free(self.data as _);
        }
    }
}

impl<'a, T: PartialEq> PartialEq<&'a [T]> for CVec<T> {
    fn eq(&self, rhs: &&[T]) -> bool {
        &self.as_ref() == rhs
    }
}

impl<'a, 'b, T: PartialEq> PartialEq<&'b [T]> for &'a CVec<T> {
    fn eq(&self, rhs: &&[T]) -> bool {
        &self.as_ref() == rhs
    }
}

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

    assert!(db.get_name().contains("rocks"));
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
        let ret = db.create_column_family(&ColumnFamilyOptions::default(), "lock");
        assert!(ret.is_ok());

        let ret = db.create_column_family(&ColumnFamilyOptions::default(), "write");
        assert!(ret.is_ok());
    }

    let opt = Options::default();
    let ret = DB::list_column_families(&opt, path);
    assert!(ret.is_ok());
    assert!(ret.as_ref().unwrap().contains(&"default".to_owned()));
    assert!(ret.as_ref().unwrap().contains(&"lock".to_owned()));
    assert!(ret.as_ref().unwrap().contains(&"write".to_owned()));

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
fn test_db_paths() {

    let opt = Options::default().map_db_options(|dbopt| {
                                                    dbopt
            .create_if_missing(true)
            .db_paths(vec!["./sample1", "./sample2"])        /* only puts sst file */
            .wal_dir("./my_wal")
                                                });

    let db = DB::open(opt, "multi");
    if db.is_err() {
        println!("err => {:?}", db.unwrap_err());
        return;
    }
    let db = db.unwrap();
    let _ = db.put(&WriteOptions::default(), b"name", b"BH1XUW")
        .unwrap();
    for i in 0..100 {
        let key = format!("test2-key-{}", i);
        let val = format!("rocksdb-value-{}", i * 10);
        let value: String = iter::repeat(val).take(10).collect::<Vec<_>>().concat();
        if i == 50 {
            let s = db.get_snapshot();
            println!("debug snapshot => {:?}", s);
        }

        db.put(&WriteOptions::default(), key.as_bytes(), value.as_bytes())
            .unwrap();
    }
}



#[test]
fn test_open_cf() {
    use tempdir::TempDir;
    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();

    let opt = Options::default().map_db_options(|db| db.create_if_missing(true));

    let ret = DB::open_with_column_families(&opt,
                                            tmp_dir.path().to_str().unwrap(),
                                            vec![ColumnFamilyDescriptor::default()]);
    assert!(ret.is_ok(), "err => {:?}", ret);
    println!("cfs => {:?}", ret);

    if let Ok((db, cfs)) = ret {
        let cf = &cfs[0];
        println!("cf name => {:?} id => {}", cf.get_name(), cf.get_id());
    }
}


#[test]
fn test_cf_lifetime() {
    use tempdir::TempDir;
    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();

    let opt = Options::default().map_db_options(|db| db.create_if_missing(true));

    let mut cf_handle = None;
    {
        let ret = DB::open_with_column_families(&opt,
                                                tmp_dir.path().to_str().unwrap(),
                                                vec![ColumnFamilyDescriptor::default()]);
        assert!(ret.is_ok(), "err => {:?}", ret);
        println!("cfs => {:?}", ret);

        if let Ok((db, mut cfs)) = ret {
            let cf = cfs.pop().unwrap();
            println!("cf name => {:?} id => {}", cf.get_name(), cf.get_id());
            cf_handle = Some(cf);
            //            unsafe {
            //                ll::rocks_db_close(db.raw());
            //            }
        }

    }

    println!("cf name => {:?}", cf_handle.unwrap().get_name());

}

#[test]
fn test_write_batch() {
    use tempdir::TempDir;
    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();

    let opt = Options::default().map_db_options(|db| db.create_if_missing(true));

    let db = DB::open(opt, tmp_dir.path().to_str().unwrap()).unwrap();

    let batch = WriteBatch::new()
        .put(b"name", b"BY1CQ")
        .delete(b"name")
        .put(b"name", b"BH1XUW")
        .put(b"site", b"github");

    let ret = db.write(WriteOptions::default(), batch);
    assert!(ret.is_ok());

    assert_eq!(db.get(ReadOptions::default(), b"name").unwrap().as_ref(),
               b"BH1XUW");
    assert_eq!(db.get(ReadOptions::default(), b"site").unwrap().as_ref(),
               b"github");
}


#[test]
fn test_compact_range() {
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
    let ret = db.compact_range(&CompactRangeOptions::default(),
                               b"test2-key-5".as_ref()..b"test2-key-9".as_ref());
    assert!(ret.is_ok());

    let ret = db.compact_range(&CompactRangeOptions::default(), ..);
    assert!(ret.is_ok());

    drop(tmp_db_dir);
}


#[test]
fn test_key_may_exist() {
    use tempdir::TempDir;
    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();

    let db = DB::open(Options::default().map_db_options(|db| db.create_if_missing(true)),
                      tmp_dir)
            .unwrap();

    db.put(&WriteOptions::default(), b"name", b"value").unwrap();

    assert!(db.key_may_exist(&ReadOptions::default(), b"name"));
    assert!(!db.key_may_exist(&ReadOptions::default(), b"name2"))
}


#[test]
fn test_db_assoc_merge() {
    use tempdir::TempDir;
    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();

    pub struct MyAssocMergeOp;

    impl AssociativeMergeOperator for MyAssocMergeOp {
        fn merge(&self,
                 key: &[u8],
                 existing_value: Option<&[u8]>,
                 value: &[u8],
                 logger: &Logger)
                 -> Option<Vec<u8>> {

            let mut ret: Vec<u8> = existing_value.map(|s| s.into()).unwrap_or(b"HEAD".to_vec());
            ret.push(b'|');
            ret.extend_from_slice(value);
            Some(ret)
        }
    }

    let db = DB::open(Options::default()
                          .map_db_options(|db| db.create_if_missing(true))
                          .map_cf_options(|cf| {
        cf.associative_merge_operator(Box::new(MyAssocMergeOp))
    }),
                      tmp_dir)
            .unwrap();

    let ret = db.merge(&WriteOptions::default(), b"name", b"value");
    let ret = db.merge(&WriteOptions::default(), b"name", b"valaerue");
    let ret = db.merge(&WriteOptions::default(), b"name", b"vzxcvalue");
    let ret = db.merge(&WriteOptions::default(), b"name", b"vasadflue");
    let ret = db.merge(&WriteOptions::default(), b"name", b"valasdfue");
    let ret = db.merge(&WriteOptions::default(), b"name", b"value");
    let ret = db.merge(&WriteOptions::default(), b"name", b"vaasdflue");
    let ret = db.merge(&WriteOptions::default(), b"name", b"vadfhlue");
    let ret = db.merge(&WriteOptions::default(), b"name", b"valadfue");
    let ret = db.merge(&WriteOptions::default(), b"name", b"valuzxve");

    let ret = db.get(&ReadOptions::default(), b"name");
    println!("after read => {:?}",
             String::from_utf8_lossy(ret.unwrap().as_ref()));
}

#[test]
fn test_db_merge() {
    use tempdir::TempDir;
    use merge_operator::{MergeOperationInput, MergeOperationOutput};

    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();

    pub struct MyMergeOp;

    impl MergeOperator for MyMergeOp {
        fn full_merge(&self,
                      merge_in: &MergeOperationInput,
                      merge_out: &mut MergeOperationOutput)
                      -> bool {
            assert_eq!(merge_in.key, b"name");
            let mut ret = b"KEY:".to_vec();
            ret.extend_from_slice(merge_in.key);
            ret.push(b'|');
            assert_eq!(merge_in.operands().len(), 3);
            for op in merge_in.operands() {
                ret.extend_from_slice(op);
                ret.push(b'+');
            }
            ret.push(b'|');
            merge_out.assign(&ret);
            true
        }
    }

    let db = DB::open(Options::default()
                          .map_db_options(|db| db.create_if_missing(true))
                          .map_cf_options(|cf| cf.merge_operator(Box::new(MyMergeOp))),
                      tmp_dir)
            .unwrap();

    let ret = db.merge(&WriteOptions::default(), b"name", b"value");
    assert!(ret.is_ok());

    let ret = db.merge(&WriteOptions::default(), b"name", b"new");
    assert!(ret.is_ok());

    let ret = db.merge(&WriteOptions::default(), b"name", b"last");
    assert!(ret.is_ok());

    let ret = db.get(&ReadOptions::default(), b"name");
    assert_eq!(ret.unwrap().as_ref(), b"KEY:name|value+new+last+|");
}


#[test]
fn test_db_merge_assign_existing_operand() {
    use tempdir::TempDir;
    use merge_operator::{MergeOperationInput, MergeOperationOutput};

    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();

    pub struct MyMergeOp;

    impl MergeOperator for MyMergeOp {
        fn full_merge(&self,
                      merge_in: &MergeOperationInput,
                      merge_out: &mut MergeOperationOutput)
                      -> bool {
            assert_eq!(merge_in.key, b"name");
            assert_eq!(merge_in.operands().len(), 6);
            let mut set = false;
            for op in merge_in.operands() {
                if op.starts_with(b"I-am-the-test") {
                    // FIXME: following not works
                    // merge_out.assign_existing_operand(op);
                    merge_out.assign(op);
                    set = true;
                    break;
                }
            }
            assert!(set);
            true
        }
    }

    let db = DB::open(Options::default()
                          .map_db_options(|db| db.create_if_missing(true))
                          .map_cf_options(|cf| cf.merge_operator(Box::new(MyMergeOp))),
                      tmp_dir)
            .unwrap();

    let ret = db.merge(&WriteOptions::default(), b"name", b"randome-key");
    assert!(ret.is_ok());
    let ret = db.merge(&WriteOptions::default(), b"name", b"asdfkjasdkf");
    assert!(ret.is_ok());
    let ret = db.merge(&WriteOptions::default(), b"name", b"sadfjalskdfjlast");
    assert!(ret.is_ok());
    let ret = db.merge(&WriteOptions::default(), b"name", b"sadfjalskdfjlast");
    assert!(ret.is_ok());
    let ret = db.merge(&WriteOptions::default(), b"name", b"I-am-the-test-233");
    assert!(ret.is_ok());
    let ret = db.merge(&WriteOptions::default(), b"name", b"I-am-not-the-test");
    assert!(ret.is_ok());
    let ret = db.get(&ReadOptions::default(), b"name");
    // println!("ret => {:?}", ret.as_ref().map(|s| String::from_utf8_lossy(s)));
    assert_eq!(ret.unwrap().as_ref(), b"I-am-the-test-233");
}

#[test]
fn test_ingest_sst_file() {
    use sst_file_writer::SstFileWriter;

    let sst_dir = ::tempdir::TempDir::new_in(".", "sst").unwrap();

    let writer = SstFileWriter::builder().build();
    writer.open(sst_dir.path().join("2333.sst")).unwrap();
    for i in 0..999 {
        let key = format!("B{:05}", i);
        let value = format!("ABCDEFGH{:03}IJKLMN", i);
        writer.add(key.as_bytes(), value.as_bytes()).unwrap();
    }
    let info = writer.finish().unwrap();
    assert_eq!(info.num_entries(), 999);

    let tmp_db_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();

    let db = DB::open(Options::default().map_db_options(|db| db.create_if_missing(true)),
                      &tmp_db_dir)
            .unwrap();

    let ret = db.ingest_external_file(&[sst_dir
                                            .path()
                                            .join("2333.sst")
                                            .to_string_lossy()
                                            .into_owned()],
                                      &IngestExternalFileOptions::default());
    assert!(ret.is_ok(), "ingest external file: {:?}", ret);

    assert!(db.get(&ReadOptions::default(), b"B00000").is_ok());
    assert_eq!(db.get(&ReadOptions::default(), b"B00000").unwrap(),
               b"ABCDEFGH000IJKLMN");
    assert_eq!(db.get(&ReadOptions::default(), b"B00998").unwrap(),
               b"ABCDEFGH998IJKLMN");
    assert!(db.get(&ReadOptions::default(), b"B00999").is_err());

    drop(sst_dir);
    drop(tmp_db_dir);
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::rocksdb::*;

    #[test]
    fn multi_get() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(Options::default().map_db_options(|db| db.create_if_missing(true)),
                          &tmp_dir)
                .unwrap();

        assert!(db.put(&Default::default(), b"a", b"1").is_ok());
        assert!(db.put(&Default::default(), b"b", b"2").is_ok());
        assert!(db.put(&Default::default(), b"c", b"3").is_ok());
        assert!(db.put(&Default::default(), b"long-key", b"long-value")
                    .is_ok());
        assert!(db.put(&Default::default(), b"e", b"5").is_ok());
        assert!(db.put(&Default::default(), b"f", b"6").is_ok());

        assert!(db.compact_range(&Default::default(), ..).is_ok());

        let ret = db.multi_get(&ReadOptions::default(),
                               &[b"a", b"b", b"c", b"f", b"long-key", b"non-exist"]);

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
        let db = DB::open(Options::default().map_db_options(|db| db.create_if_missing(true)),
                          &tmp_dir)
                .unwrap();

        let def = db.default_column_family();
        let cf1 = db.create_column_family(&Default::default(), "db1").unwrap();
        let cf2 = db.create_column_family(&Default::default(), "db2").unwrap();
        let cf3 = db.create_column_family(&Default::default(), "db3").unwrap();
        let cf4 = db.create_column_family(&Default::default(), "db4").unwrap();

        assert!(db.put_cf(&WriteOptions::default(), &def, b"AA", b"aa")
                    .is_ok());
        assert!(db.put_cf(&WriteOptions::default(), &cf1, b"BB", b"bb")
                    .is_ok());
        assert!(db.put_cf(&WriteOptions::default(), &cf2, b"CC", b"cc")
                    .is_ok());
        assert!(db.put_cf(&WriteOptions::default(), &cf3, b"DD", b"dd")
                    .is_ok());
        assert!(db.put_cf(&WriteOptions::default(), &cf4, b"EE", b"ee")
                    .is_ok());

        assert!(db.compact_range(&Default::default(), ..).is_ok());

        let ret = db.multi_get_cf(&ReadOptions::default(),
                                  &[&def, &cf1, &cf2, &cf3, &cf4, &def],
                                  &[b"AA", b"BB", b"CC", b"DD", b"EE", b"233"]);

        assert_eq!(ret[0].as_ref().unwrap(), b"aa".as_ref());
        assert_eq!(ret[2].as_ref().unwrap(), b"cc".as_ref());
        assert_eq!(ret[4].as_ref().unwrap(), b"ee".as_ref());
        assert!(ret[5].as_ref().unwrap_err().is_not_found());

        // mem::forget(def);
    }

    #[test]
    fn key_may_exist() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(Options::default().map_db_options(|db| db.create_if_missing(true)),
                          &tmp_dir)
                .unwrap();

        assert!(db.put(&Default::default(), b"long-key", b"long-value")
                    .is_ok());
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
        let db = DB::open(Options::default().map_db_options(|db| db.create_if_missing(true)),
                          &tmp_dir)
            .unwrap();

        assert!(db.put(&Default::default(), b"long-key", vec![b'A'; 1024 * 1024].as_ref())
                .is_ok());

        let cf1 = db.create_column_family(&Default::default(), "db1").unwrap();

        assert!(db.compact_range(&Default::default(), ..).is_ok());

        let snap = db.get_snapshot();
        assert_eq!(db.get_property("rocksdb.num-snapshots"), Some("1".to_string()));

        // dump status
        println!("stats => {}", db.get_property("rocksdb.stats").unwrap());
        assert_eq!(db.get_int_property("rocksdb.num-snapshots"), Some(1));

        assert!(db.put(&Default::default(), b"long-key2", vec![b'A'; 1024 * 1024].as_ref())
                .is_ok());

        assert!(db.put_cf(&Default::default(), &cf1, b"long-key2", vec![b'A'; 1024 * 1024].as_ref())
                .is_ok());

        assert!(db.get_int_property("rocksdb.size-all-mem-tables").unwrap() < 2 * 1024 * 1024);

        assert!(db.get_aggregated_int_property("rocksdb.size-all-mem-tables").unwrap() > 2 * 1024 * 1024);
    }

    #[test]
    fn misc_functions() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(Options::default()
                          .map_db_options(|db| db.create_if_missing(true))
                          .map_cf_options(|cf| cf.disable_auto_compactions(true)),
                          &tmp_dir)
            .unwrap();

        assert!(db.put(&Default::default(), b"long-key", vec![b'A'; 1024 * 1024].as_ref())
                .is_ok());
        assert!(db.put(&Default::default(), b"a", b"1").is_ok());
        assert!(db.put(&Default::default(), b"b", b"2").is_ok());
        assert!(db.put(&Default::default(), b"c", b"3").is_ok());

        assert!(db.compact_range(&Default::default(), ..).is_ok());

        assert!(db.pause_background_work().is_ok());
        assert!(db.continue_background_work().is_ok());

        assert!(db.enable_auto_compaction(&[&db.default_column_family()]).is_ok());

        assert_eq!(db.number_levels(), 7); // default
        assert_eq!(db.max_mem_compaction_level(), 0); // TODO: wtf
        assert_eq!(db.level0_stop_write_trigger(), 36); // default

        assert!(db.get_db_identity().is_ok());
        println!("id => {:?}", db.get_db_identity());
    }

    #[test]
    fn flush() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(Options::default()
                          .map_db_options(|db| db.create_if_missing(true))
                          .map_cf_options(|cf| cf.disable_auto_compactions(true)),
                          &tmp_dir)
            .unwrap();

        assert_eq!(db.get_latest_sequence_number(), 0);

        assert!(db.put(&Default::default(), b"long-key", vec![b'A'; 1024 * 1024].as_ref())
                .is_ok());
        assert!(db.put(&Default::default(), b"a", b"1").is_ok());
        assert!(db.put(&Default::default(), b"b", b"2").is_ok());
        assert!(db.put(&Default::default(), b"c", b"3").is_ok());

        assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
        assert!(db.sync_wal().is_ok());

        // 5th transaction
        assert_eq!(db.get_latest_sequence_number(), 4);
    }
}
