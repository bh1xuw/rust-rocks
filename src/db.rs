
use std::mem;
use std::ffi::{CStr, CString};
use std::os::raw::{c_int, c_char};
use std::ptr;
use std::iter;
use std::str;
use std::slice;
use std::rc::{Rc, Weak};
use std::ops;
use std::fmt;
use std::iter::IntoIterator;
use std::marker::PhantomData;

use rocks_sys as ll;

use status::Status;
use comparator::Comparator;
use options::{Options, DBOptions, ColumnFamilyOptions, ReadOptions, WriteOptions};
use table_properties::TableProperties;
use snapshot::Snapshot;

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
    db: Rc<DBContext<'b>>,      // 'b out lives 'a
    _marker: PhantomData<&'a ()>,
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
        unsafe {
            ll::rocks_column_family_handle_get_id(self.raw)
        }
    }

    /// Fills "*desc" with the up-to-date descriptor of the column family
    /// associated with this handle. Since it fills "*desc" with the up-to-date
    /// information, this call might internally lock and release DB mutex to
    /// access the up-to-date CF options.  In addition, all the pointer-typed
    /// options cannot be referenced any longer than the original options exist.
    ///
    /// Note that this function is not supported in RocksDBLite.
    pub fn get_descriptor(&self, desc: &mut ColumnFamilyDescriptor) -> Result<ColumnFamilyDescriptor, Status> {
        unimplemented!()
    }

    /// Returns the comparator of the column family associated with the
    /// current handle.
    pub fn get_comparator(&self) -> Comparator {
        unimplemented!()
    }
}

impl<'a, 'b> Drop for ColumnFamilyHandle<'a, 'b> {
    fn drop(&mut self) {
        // TODO: use rocks_db_destroy_column_family_handle
        //unsafe { ll::rocks_column_family_handle_destroy(self.raw) }
        unsafe {
            let mut status = mem::zeroed();
            ll::rocks_db_destroy_column_family_handle(self.db.raw, self.raw(), &mut status);
            assert!(status.code == 0);
        }
    }
}

/// A range of keys
pub struct Range<'a> {
    /// Included in the range
    start: &'a [u8],
    /// Not included in the range
    limit: &'a [u8],
}


impl<'a> Range<'a> {
    pub fn new(s: &'a [u8], l: &'a [u8]) -> Range<'a> {
        Range {
            start: s,
            limit: l,
        }
    }
}


pub struct DBContext<'a> {
    raw: *mut ll::rocks_db_t,
    _marker: PhantomData<&'a ()>
}

impl<'a> Drop for DBContext<'a> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            ll::rocks_db_close(self.raw);
        }
    }
}


/// A DB is a persistent ordered map from keys to values.
/// A DB is safe for concurrent access from multiple threads without
/// any external synchronization.
pub struct DB<'a> {
    context: Rc<DBContext<'a>>,
}

impl<'a> fmt::Debug for DB<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DB({:?})", self.context.raw)
    }
}


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

    /// Open the database with the specified "name".
    /// Stores a pointer to a heap-allocated database in *dbptr and returns
    /// OK on success.
    /// Stores nullptr in *dbptr and returns a non-OK status on error.
    /// Caller should delete *dbptr when it is no longer needed.
    pub fn open<'b>(options: &Options, name: &str) -> Result<DB<'b>, Status> {
        unsafe {
            let opt = options.raw();
            let dbname = CString::new(name).unwrap();
            let mut status = mem::uninitialized::<ll::rocks_status_t>();
            let db_ptr = ll::rocks_db_open(opt, dbname.as_ptr(), &mut status);
            if status.code == 0 {
                Ok(DB::from_ll(db_ptr))
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }

    /// Open DB with column families.
    /// db_options specify database specific options
    /// column_families is the vector of all column families in the database,
    /// containing column family name and options. You need to open ALL column
    /// families in the database. To get the list of column families, you can use
    /// ListColumnFamilies(). Also, you can open only a subset of column families
    /// for read-only access.
    /// The default column family name is 'default' and it's stored
    /// in rocksdb::kDefaultColumnFamilyName.
    /// If everything is OK, handles will on return be the same size
    /// as column_families --- handles[i] will be a handle that you
    /// will use to operate on column family column_family[i].
    /// Before delete DB, you have to close All column families by calling
    /// DestroyColumnFamilyHandle() with all the handles.
    pub fn open_with_column_families<'b, 'c: 'b, CF: Into<ColumnFamilyDescriptor>>(
        options: &Options, name: &str, column_families: Vec<CF>) ->
        Result<(DB<'c>, Vec<ColumnFamilyHandle<'b, 'c>>), Status> {
        unsafe {
            let opt = options.raw();
            let mut status = mem::uninitialized::<ll::rocks_status_t>();
            let dbname = CString::new(name).unwrap();

            let cfs: Vec<ColumnFamilyDescriptor> = column_families.into_iter().map(|desc| desc.into()).collect();

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

            let db_ptr = ll::rocks_db_open_column_families(
                options.raw(),
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
                    cfhandles.into_iter().map(move |p| {
                        ColumnFamilyHandle {
                            raw: p,
                            db: db_ctx.clone(),
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
    /// that modify data, like put/delete, will return error.
    /// If the db is opened in read only mode, then no compactions
    /// will happen.
    ///
    /// Not supported in ROCKSDB_LITE, in which case the function will
    /// return Status::NotSupported.
    pub fn open_for_readonly<'b>(options: &Options, name: &str, error_if_log_file_exist: bool) -> Result<DB<'b>, Status> {
        unsafe {
            let dbname = CString::new(name).unwrap();
            let mut status = mem::uninitialized::<ll::rocks_status_t>();
            let db_ptr = ll::rocks_db_open_for_read_only(
                options.raw(), dbname.as_ptr(), error_if_log_file_exist as u8, &mut status);
            if status.code == 0 {
                Ok(DB::from_ll(db_ptr))
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }


    /// ListColumnFamilies will open the DB specified by argument name
    /// and return the list of all column families in that DB
    /// through column_families argument. The ordering of
    /// column families in column_families is unspecified.
    pub fn list_column_families(options: &Options, name: &str) -> Result<Vec<String>, Status> {
        unsafe {
            let dbname = CString::new(name).unwrap();
            let mut status = mem::uninitialized::<ll::rocks_status_t>();
            let mut lencfs = 0;
            let cfs = ll::rocks_db_list_column_families(
                options.raw(),
                dbname.as_ptr(),
                &mut lencfs,
                &mut status);
            if status.code == 0 {
                if lencfs == 0 { Ok(vec![])}
                else {
                    let mut ret = Vec::with_capacity(lencfs);
                    for i in 0..lencfs {
                        ret.push(CStr::from_ptr(*cfs.offset(i as isize)).to_str().unwrap().to_string());
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
                                column_family_name: &str) -> Result<ColumnFamilyHandle, Status> {
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
                    _marker: PhantomData,
                })
            }else {
                Err(Status::from_ll(&status))
            }
        }
    }

    // Set the database entry for "key" to "value".
    // If "key" already exists, it will be overwritten.
    // Returns OK on success, and a non-OK status on error.
    // Note: consider setting options.sync = true.
    pub fn put_cf(&self, options: &WriteOptions, column_family: &ColumnFamilyHandle,
               key: &[u8], value: &[u8]) -> Result<(), Status> {
        unsafe {
            let mut status = mem::uninitialized::<ll::rocks_status_t>();
            // since rocksdb::DB::put without cf is for compatibility
            ll::rocks_db_put_cf(
                self.raw(),
                options.raw(),
                column_family.raw(),
                key.as_ptr() as _, key.len(),
                value.as_ptr() as _, value.len(),
                &mut status);
            if status.code == 0 {
                Ok(())
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }

    pub fn put(&self, options: &WriteOptions,
               key: &[u8], value: &[u8]) -> Result<(), Status> {
        unsafe {
            let mut status = mem::uninitialized::<ll::rocks_status_t>();
            ll::rocks_db_put(
                self.raw(),
                options.raw(),
                key.as_ptr() as _, key.len(),
                value.as_ptr() as _, value.len(),
                &mut status);
            if status.code == 0 {
                Ok(())
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }

/*
    pub fn put_slice(&self, options: &WriteOptions,
               key: &[u8], value: &[u8]) -> Result<(), Status> {
        unsafe {
            let mut status = mem::uninitialized::<ll::rocks_status_t>();
            ll::rocks_db_put_slice(
                self.raw,
                options.raw(),
                mem::transmute::<&&[u8], *const ll::Slice>(&key),
                mem::transmute::<&&[u8], *const ll::Slice>(&value),
                &mut status);
            if status.code == 0 {
                Ok(())
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }
     */
    // delete

    // merge

    // write

    // If the database contains an entry for "key" store the
    // corresponding value in *value and return OK.
    //
    // If there is no entry for "key" leave *value unchanged and return
    // a status for which Status::IsNotFound() returns true.
    //
    // May return some other Status on an error.
    pub fn get(&self, options: &ReadOptions, key: &[u8]) -> Result<CVec<u8>, Status> {
        unsafe {
            let mut status = mem::zeroed::<ll::rocks_status_t>();
            let mut vallen = 0_usize;
            let ptr = ll::rocks_db_get(self.raw(), options.raw(),
                                       key.as_ptr() as _, key.len(),
                                       &mut vallen, &mut status);

            if status.code == 0 {
                Ok(CVec::from_raw_parts(ptr as *mut u8, vallen))
            } else {
                Err(Status::from_ll(&status))
            }
        }
    }

    pub fn get_cf(&self, options: &ReadOptions, column_family: &ColumnFamilyHandle,
                  key: &[u8]) -> Result<CVec<u8>, Status> {
        unsafe {
            let mut status = mem::zeroed::<ll::rocks_status_t>();
            let mut vallen = 0_usize;
            let ptr = ll::rocks_db_get_cf(self.raw(), options.raw(),
                                          column_family.raw(),
                                          key.as_ptr() as _, key.len(),
                                          &mut vallen, &mut status);

            if status.code == 0 {
                Ok(CVec::from_raw_parts(ptr as *mut u8, vallen))
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
}


pub struct CVec<T> {
    data: *mut T,
    len: usize,
}

impl<T> CVec<T> {
    pub unsafe fn from_raw_parts(p: *mut T, len: usize) -> CVec<T> {
        CVec {
            data: p,
            len: len,
        }
    }

}

impl CVec<u8> {
    pub fn to_str(&self) -> Result<&str, str::Utf8Error> {
        str::from_utf8(self)
    }
}

impl<T: fmt::Debug> fmt::Debug for CVec<T>{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            slice::from_raw_parts(self.data, self.len).fmt(f)
        }
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



#[test]
fn it_works() {
    use super::advanced_options::CompactionPri;
    use tempdir::TempDir;

    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();
    let path = tmp_dir.path().to_str().unwrap();

    // staircase style config
    let opt = Options::default()
        .map_db_options(|dbopt| {
            dbopt
                .create_if_missing(true)
        })
        .map_cf_options(|cfopt| {
            cfopt
                .compaction_pri(CompactionPri::MinOverlappingRatio)
        })
        .optimize_for_small_db();
    let db = DB::open(&opt, path);
    assert!(db.is_ok(), "err => {:?}", db);
    let db = db.unwrap();
    let cfhandle = db.create_column_family(&ColumnFamilyOptions::default(),
                                           "lock");
    println!("cf => {:?}", cfhandle);
}

#[test]
fn test_open_for_readonly() {
    use tempdir::TempDir;

    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();
    let path = tmp_dir.path().to_str().unwrap();

    {
        let opt = Options::default()
            .map_db_options(|opt| opt.create_if_missing(true));
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
        let opt = Options::default()
            .map_db_options(|opt| opt.create_if_missing(true));
        let db = DB::open(&opt, path);
        assert!(db.is_ok());

        let db = db.unwrap();
        let ret = db.create_column_family(&ColumnFamilyOptions::default(),
                                          "lock");
        assert!(ret.is_ok());

        let ret = db.create_column_family(&ColumnFamilyOptions::default(),
                                          "write");
        assert!(ret.is_ok());
    }

    let opt = Options::default();
    let ret = DB::list_column_families(&opt, path);
    assert!(ret.is_ok());
    assert!(ret.as_ref().unwrap().contains(&"default".to_owned()));
    assert!(ret.as_ref().unwrap().contains(&"lock".to_owned()));
    assert!(ret.as_ref().unwrap().contains(&"write".to_owned()));
}

#[test]
fn test_db_get() {
    use tempdir::TempDir;

    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();
    let path = tmp_dir.path().to_str().unwrap();

    {

        let opt = Options::default()
        .map_db_options(|dbopt| {
            dbopt
                .create_if_missing(true)
        });

        let db = DB::open(&opt, path);
        assert!(db.is_ok(), "err => {:?}", db.as_ref().unwrap_err());
        let db = db.unwrap();
        let _ = db.put(&WriteOptions::default(),
                       b"name", b"BH1XUW");
    }

    let db = DB::open(&Default::default(), path).unwrap();
    let val = db.get(&ReadOptions::default(),
                     b"name");
    assert_eq!(val.unwrap().as_ref(), b"BH1XUW");
}


#[test]
fn test_db_paths() {

    let opt = Options::default()
        .map_db_options(|dbopt| {
            dbopt
                .create_if_missing(true)
                .db_paths(vec!["./sample1", // only puts sst file
                               "./sample2"])
                .wal_dir("./my_wal")
        });

    let db = DB::open(&opt, "multi");
    if db.is_err() {
        println!("err => {:?}", db.unwrap_err());
        return ;
    }
    let db = db.unwrap();
    let _ = db.put(&Default::default(),
                   b"name", b"BH1XUW").unwrap();
    for i in 0..1000 {
            let key = format!("test2-key-{}", i);
            let val = format!("rocksdb-value-{}", i*10);
            let value: String = iter::repeat(val)
                .take(100)
                .collect::<Vec<_>>()
                .concat();

            db.put(&WriteOptions::default(),
                   key.as_bytes(), value.as_bytes()).unwrap();
        }

}



#[test]
fn test_open_cf() {
    use tempdir::TempDir;
    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();

    let opt = Options::default()
        .map_db_options(|db| {
            db.create_if_missing(true)
        });

    let ret = DB::open_with_column_families(&opt, tmp_dir.path().to_str().unwrap(),
                                            vec![ColumnFamilyDescriptor::default()]);
    assert!(ret.is_ok(), "err => {:?}", ret);
    println!("cfs => {:?}", ret);

    if let Ok((db, cfs)) = ret {
        let cf = &cfs[0];
        println!("cf name => {:?} id => {}", cf.get_name(), cf.get_id());
    }
}
