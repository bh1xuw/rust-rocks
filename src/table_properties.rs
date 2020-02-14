//! contains a bunch of read-only properties of its associated
//! table.

use std::u32;
use std::slice;
use std::str;
use std::fmt;
use std::mem;
use std::marker::PhantomData;
use std::ops;
use std::os::raw::{c_char, c_void};

use rocks_sys as ll;

use types::SequenceNumber;
use to_raw::{FromRaw, ToRaw};

pub const UNKNOWN_COLUMN_FAMILY_ID: u32 = u32::MAX;

/// A collections of table properties objects, where
///  key: is the table's file name.
///  value: the table properties object of the given table.
pub struct TablePropertiesCollection {
    // std::unordered_map<std::string, std::shared_ptr<const TableProperties>>
    raw: *mut ll::rocks_table_props_collection_t,
}

impl fmt::Debug for TablePropertiesCollection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TablePropertiesCollection {{{} items}}", self.len())
    }
}

impl FromRaw<ll::rocks_table_props_collection_t> for TablePropertiesCollection {
    unsafe fn from_ll(raw: *mut ll::rocks_table_props_collection_t) -> Self {
        TablePropertiesCollection { raw: raw }
    }
}

impl Drop for TablePropertiesCollection {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_table_props_collection_destroy(self.raw);
        }
    }
}

impl TablePropertiesCollection {
    pub fn len(&self) -> usize {
        unsafe { ll::rocks_table_props_collection_size(self.raw) as usize }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> TablePropertiesCollectionIter {
        TablePropertiesCollectionIter {
            raw: unsafe { ll::rocks_table_props_collection_iter_create(self.raw) },
            size: self.len(),
            at_end: self.is_empty(),
            _marker: PhantomData,
        }
    }
}

#[doc(hidden)]
pub struct TablePropertiesCollectionIter<'a> {
    raw: *mut ll::rocks_table_props_collection_iter_t,
    size: usize,
    at_end: bool,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Drop for TablePropertiesCollectionIter<'a> {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_table_props_collection_iter_destroy(self.raw);
        }
    }
}

impl<'a> Iterator for TablePropertiesCollectionIter<'a> {
    type Item = (&'a str, TableProperties<'a>);

    fn next(&mut self) -> Option<(&'a str, TableProperties<'a>)> {
        if self.raw.is_null() || self.at_end {
            None
        } else {
            let mut key_len = 0;
            unsafe {
                let key_ptr = ll::rocks_table_props_collection_iter_key(self.raw, &mut key_len);
                let prop = TableProperties::from_ll(ll::rocks_table_props_collection_iter_value(self.raw));
                self.at_end = ll::rocks_table_props_collection_iter_next(self.raw) == 0;
                let key = str::from_utf8_unchecked(slice::from_raw_parts(key_ptr as *const u8, key_len));
                Some((key, prop))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.size, Some(self.size))
    }
}


/// Other than basic table properties, each table may also have the user
/// collected properties.
///
/// The value of the user-collected properties are encoded as raw bytes --
/// users have to interprete these values by themselves.
///
/// Rust: wraps raw c pointer, and exposes as a `{String => Vec<u8>}` map
///
/// Common properties:
///
/// * `rocksdb.block.based.table.index.type`
/// * `rocksdb.block.based.table.prefix.filtering`
/// * `rocksdb.block.based.table.whole.key.filtering`
/// * `rocksdb.deleted.keys`
/// * `rocksdb.merge.operands`
#[repr(C)]
pub struct UserCollectedProperties {
    // *std::map<std::string, std::string>
    raw: *mut c_void,
}

impl fmt::Debug for UserCollectedProperties {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UserCollectedProperties {{{} items}}", self.len())
    }
}

impl ToRaw<ll::rocks_user_collected_props_t> for UserCollectedProperties {
    fn raw(&self) -> *mut ll::rocks_user_collected_props_t {
        unsafe { mem::transmute(self as *const UserCollectedProperties as *mut c_void) }
    }
}

impl UserCollectedProperties {
    pub fn insert(&mut self, key: &str, value: &[u8]) {
        unsafe {
            ll::rocks_user_collected_props_insert(
                self.raw(),
                key.as_ptr() as *const _,
                key.len(),
                value.as_ptr() as *const _,
                value.len(),
            );
        }
    }

    pub fn len(&self) -> usize {
        unsafe { ll::rocks_user_collected_props_size(self.raw()) as usize }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> UserCollectedPropertiesIter {
        UserCollectedPropertiesIter {
            raw: unsafe { ll::rocks_user_collected_props_iter_create(self.raw()) },
            size: self.len(),
            at_end: self.is_empty(),
            _marker: PhantomData,
        }
    }
}

impl<'a> ops::Index<&'a str> for UserCollectedProperties {
    type Output = [u8];
    fn index(&self, index: &'a str) -> &[u8] {
        let mut size = 0;
        unsafe {
            let val_ptr = ll::rocks_user_collected_props_at(
                self.raw(),
                index.as_bytes().as_ptr() as *const c_char,
                index.len(),
                &mut size,
            );
            if val_ptr.is_null() {
                panic!("key not found {:?}", index);
            }
            slice::from_raw_parts(val_ptr as *const u8, size)
        }
    }
}

/// Rust Iterator interface for `UserCollectedProperties`
pub struct UserCollectedPropertiesIter<'a> {
    raw: *mut ll::rocks_user_collected_props_iter_t,
    size: usize,
    at_end: bool,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Drop for UserCollectedPropertiesIter<'a> {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_user_collected_props_iter_destroy(self.raw);
        }
    }
}

impl<'a> Iterator for UserCollectedPropertiesIter<'a> {
    type Item = (&'a str, &'a [u8]);

    fn next(&mut self) -> Option<(&'a str, &'a [u8])> {
        if self.raw.is_null() || self.at_end {
            None
        } else {
            let mut key_len = 0;
            let mut value_len = 0;
            unsafe {
                let key_ptr = ll::rocks_user_collected_props_iter_key(self.raw, &mut key_len);
                let value_ptr = ll::rocks_user_collected_props_iter_value(self.raw, &mut value_len);
                self.at_end = ll::rocks_user_collected_props_iter_next(self.raw) == 0;
                let key = str::from_utf8_unchecked(slice::from_raw_parts(key_ptr as *const u8, key_len));
                let value = slice::from_raw_parts(value_ptr as *const u8, value_len);
                Some((key, value))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.size, Some(self.size))
    }
}

/// `TableProperties` contains a bunch of read-only properties of its associated
/// table.
#[repr(C)]
pub struct TableProperties<'a> {
    raw: *mut ll::rocks_table_props_t,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Drop for TableProperties<'a> {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_table_props_destroy(self.raw);
        }
    }
}

impl<'a> FromRaw<ll::rocks_table_props_t> for TableProperties<'a> {
    unsafe fn from_ll(raw: *mut ll::rocks_table_props_t) -> Self {
        TableProperties {
            raw: raw,
            _marker: PhantomData,
        }
    }
}

impl<'a> fmt::Display for TableProperties<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ret = String::new();
        unsafe {
            ll::rocks_table_props_to_string(self.raw, &mut ret as *mut String as *mut c_void);
        }
        write!(f, "{}", ret)
    }
}

impl<'a> fmt::Debug for TableProperties<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TableProperties {{{:?}}}", self.to_string())
    }
}

impl<'a> TableProperties<'a> {
    /// the total size of all data blocks.
    pub fn data_size(&self) -> u64 {
        unsafe { ll::rocks_table_props_get_data_size(self.raw) }
    }
    /// the size of index block.
    pub fn index_size(&self) -> u64 {
        unsafe { ll::rocks_table_props_get_index_size(self.raw) }
    }
    /// the size of filter block.
    pub fn filter_size(&self) -> u64 {
        unsafe { ll::rocks_table_props_get_filter_size(self.raw) }
    }
    /// total raw key size
    pub fn raw_key_size(&self) -> u64 {
        unsafe { ll::rocks_table_props_get_raw_key_size(self.raw) }
    }
    /// total raw value size
    pub fn raw_value_size(&self) -> u64 {
        unsafe { ll::rocks_table_props_get_raw_value_size(self.raw) }
    }
    /// the number of blocks in this table
    pub fn num_data_blocks(&self) -> u64 {
        unsafe { ll::rocks_table_props_get_num_data_blocks(self.raw) }
    }
    /// the number of entries in this table
    pub fn num_entries(&self) -> u64 {
        unsafe { ll::rocks_table_props_get_num_entries(self.raw) }
    }
    /// format version, reserved for backward compatibility
    pub fn format_version(&self) -> u64 {
        unsafe { ll::rocks_table_props_get_format_version(self.raw) }
    }
    /// If 0, key is variable length. Otherwise number of bytes for each key.
    pub fn fixed_key_len(&self) -> u64 {
        unsafe { ll::rocks_table_props_get_format_version(self.raw) }
    }
    /// ID of column family for this SST file, corresponding to the CF identified
    /// by column_family_name.
    pub fn column_family_id(&self) -> u32 {
        unsafe { ll::rocks_table_props_get_column_family_id(self.raw) }
    }

    /// Name of the column family with which this SST file is associated.
    /// If column family is unknown, `column_family_name` will be an empty string.
    pub fn column_family_name(&self) -> Option<&str> {
        let mut len = 0;
        unsafe {
            let ptr = ll::rocks_table_props_get_column_family_name(self.raw, &mut len);
            if len != 0 {
                Some(str::from_utf8_unchecked(slice::from_raw_parts(ptr as *const _, len)))
            } else {
                None
            }
        }
    }

    /// The name of the filter policy used in this table.
    /// If no filter policy is used, `filter_policy_name` will be an empty string.
    pub fn filter_policy_name(&self) -> Option<&str> {
        let mut len = 0;
        unsafe {
            let ptr = ll::rocks_table_props_get_filter_policy_name(self.raw, &mut len);
            if len != 0 {
                Some(str::from_utf8_unchecked(slice::from_raw_parts(ptr as *const _, len)))
            } else {
                None
            }
        }
    }

    /// The name of the comparator used in this table.
    pub fn comparator_name(&self) -> &str {
        let mut len = 0;
        unsafe {
            let ptr = ll::rocks_table_props_get_comparator_name(self.raw, &mut len);
            str::from_utf8_unchecked(slice::from_raw_parts(ptr as *const _, len))
        }
    }

    /// The name of the merge operator used in this table.
    /// If no merge operator is used, `merge_operator_name` will be "nullptr".
    pub fn merge_operator_name(&self) -> Option<&str> {
        let mut len = 0;
        unsafe {
            let ptr = ll::rocks_table_props_get_merge_operator_name(self.raw, &mut len);
            if len != 0 {
                Some(str::from_utf8_unchecked(slice::from_raw_parts(ptr as *const _, len)))
            } else {
                None
            }
        }
    }

    /// The name of the prefix extractor used in this table
    /// If no prefix extractor is used, `prefix_extractor_name` will be "nullptr".
    pub fn prefix_extractor_name(&self) -> Option<&str> {
        let mut len = 0;
        unsafe {
            let ptr = ll::rocks_table_props_get_prefix_extractor_name(self.raw, &mut len);
            if len != 0 {
                Some(str::from_utf8_unchecked(slice::from_raw_parts(ptr as *const _, len)))
            } else {
                None
            }
        }
    }

    /// The names of the property collectors factories used in this table
    /// separated by commas
    /// {collector_name[1]},{collector_name[2]},{collector_name[3]} ..
    /// or []
    pub fn property_collectors_names(&self) -> &str {
        let mut len = 0;
        unsafe {
            let ptr = ll::rocks_table_props_get_property_collectors_names(self.raw, &mut len);
            str::from_utf8_unchecked(slice::from_raw_parts(ptr as *const _, len))
        }
    }

    /// The compression algo used to compress the SST files.
    pub fn compression_name(&self) -> &str {
        let mut len = 0;
        unsafe {
            let ptr = ll::rocks_table_props_get_compression_name(self.raw, &mut len);
            str::from_utf8_unchecked(slice::from_raw_parts(ptr as *const _, len))
        }
    }

    /// user collected properties
    pub fn user_collected_properties(&self) -> &UserCollectedProperties {
        unsafe {
            let raw_ptr = ll::rocks_table_props_get_user_collected_properties(self.raw);
            &*(raw_ptr as *const UserCollectedProperties)
        }
    }

    pub fn readable_properties(&self) -> &UserCollectedProperties {
        unsafe {
            let raw_ptr = ll::rocks_table_props_get_readable_properties(self.raw);
            &*(raw_ptr as *const UserCollectedProperties)
        }
    }
}

/// Different kinds of entry type of a table
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum EntryType {
    EntryPut,
    EntryDelete, // value will be empty
    EntrySingleDelete, // value will be empty
    EntryMerge,
    EntryOther,
}

/// `TablePropertiesCollector` provides the mechanism for users to collect
/// their own properties that they are interested in. This class is essentially
/// a collection of callback functions that will be invoked during table
/// building. It is construced with `TablePropertiesCollectorFactory`. The methods
/// don't need to be thread-safe, as we will create exactly one
/// `TablePropertiesCollector` object per table and then call it sequentially
pub trait TablePropertiesCollector {
    /// AddUserKey() will be called when a new key/value pair is inserted into the
    /// table.
    ///
    /// @params `key`    the user key that is inserted into the table.
    /// @params `value`  the value that is inserted into the table.
    fn add_user_key(&mut self, key: &[u8], value: &[u8], type_: EntryType, seq: SequenceNumber, file_size: u64);

    /// Finish() will be called when a table has already been built and is ready
    /// for writing the properties block.
    ///
    /// @params `properties`  User will add their collected statistics to
    /// `properties`.
    fn finish(&mut self, properties: &mut UserCollectedProperties);

    /// The name of the properties collector can be used for debugging purpose.
    fn name(&self) -> &str {
        "RustTablePropertiesCollector\0"
    }

    /// Return the human-readable properties, where the key is property name and
    /// the value is the human-readable form of value.
    ///
    /// TODO:
    fn readable_properties(&self) -> Vec<(String, String)> {
        unimplemented!()
    }

    /// Return whether the output file should be further compacted
    fn need_compact(&self) -> bool {
        false
    }
}

/// Context of a table properties collector
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Context {
    pub column_family_id: u32,
}

/// Constructs `TablePropertiesCollector`. Internals create a new
/// `TablePropertiesCollector` for each new table
pub trait TablePropertiesCollectorFactory {
    /// has to be thread-safe
    fn new_collector(&mut self, context: Context) -> Box<dyn TablePropertiesCollector>;

    /// The name of the properties collector can be used for debugging purpose.
    fn name(&self) -> &str {
        "RustTablePropertiesCollectorFactory\0"
    }
}

#[doc(hidden)]
pub mod c {
    use std::mem;
    use std::os::raw::{c_char, c_int, c_uchar};

    use super::*;

    #[no_mangle]
    pub unsafe extern "C" fn rust_table_props_collector_add_user_key(
        c: *mut (),
        key: &&[u8],
        value: &&[u8],
        type_: c_int,
        seq: u64,
        file_size: u64,
    ) {
        assert!(!c.is_null());
        let collector = c as *mut Box<dyn TablePropertiesCollector>;
        (*collector).add_user_key(key, value, mem::transmute(type_), SequenceNumber(seq), file_size);
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_table_props_collector_finish(c: *mut (), props: *mut UserCollectedProperties) {
        assert!(!c.is_null());
        let collector = c as *mut Box<dyn TablePropertiesCollector>;
        props.as_mut().map(|p| (*collector).finish(p));
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_table_props_collector_name(c: *mut ()) -> *const c_char {
        assert!(!c.is_null());
        let collector = c as *mut Box<dyn TablePropertiesCollector>;
        (*collector).name().as_ptr() as *const _
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_table_props_collector_need_compact(c: *mut ()) -> c_uchar {
        assert!(!c.is_null());
        let collector = c as *mut Box<dyn TablePropertiesCollector>;
        (*collector).need_compact() as c_uchar
    }

    // yes, will be called :)
    #[no_mangle]
    pub unsafe extern "C" fn rust_table_props_collector_drop(f: *mut ()) {
        assert!(!f.is_null());
        let filter = f as *mut Box<dyn TablePropertiesCollector>;
        Box::from_raw(filter);
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_table_props_collector_factory_new_collector(
        f: *mut (),
        cf_id: u32,
    ) -> *mut Box<dyn TablePropertiesCollector> {
        assert!(!f.is_null());
        let factory = f as *mut Box<dyn TablePropertiesCollectorFactory>;
        let collector = (*factory).new_collector(Context { column_family_id: cf_id });
        Box::into_raw(Box::new(collector))
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_table_props_collector_factory_name(f: *mut ()) -> *const c_char {
        assert!(!f.is_null());
        let factory = f as *mut Box<dyn TablePropertiesCollectorFactory>;
        (*factory).name().as_ptr() as *const _
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_table_props_collector_factory_drop(f: *mut ()) {
        assert!(!f.is_null());
        let filter = f as *mut Box<dyn TablePropertiesCollectorFactory>;
        Box::from_raw(filter);
    }
}


#[cfg(test)]
mod tests {
    use std::iter;
    use std::time;

    use super::*;
    use super::super::rocksdb::*;

    #[derive(Default)]
    pub struct MyTblPropsCollector {
        counter: u32,
    }

    impl TablePropertiesCollector for MyTblPropsCollector {
        fn add_user_key(&mut self, key: &[u8], value: &[u8], type_: EntryType, seq: SequenceNumber, file_size: u64) {
            // self.counter += 1;
            // println!("{:?} {:?} {:?} => {:?}", type_, seq, key, value);
        }

        fn finish(&mut self, props: &mut UserCollectedProperties) {
            props.insert("hello", b"world");
            props.insert("sample_key", b"sample_value");
            props.insert("test.counter", format!("{}", self.counter).as_bytes());
        }
    }

    pub struct MyTblPropsCollectorFactory;

    impl TablePropertiesCollectorFactory for MyTblPropsCollectorFactory {
        fn new_collector(&mut self, context: Context) -> Box<dyn TablePropertiesCollector> {
            Box::new(MyTblPropsCollector {
                counter: time::SystemTime::now()
                    .duration_since(time::UNIX_EPOCH)
                    .unwrap()
                    .subsec_nanos(),
            })
        }
    }


    #[test]
    fn table_properties() {
        let tmp_dir = ::tempdir::TempDir::new_in("", "rocks").unwrap();
        let db = DB::open(
            Options::default()
                .map_db_options(|db| db.create_if_missing(true))
                .map_cf_options(|cf| {
                    cf.disable_auto_compactions(true)
                        .table_properties_collector_factory(Box::new(MyTblPropsCollectorFactory))
                }),
            &tmp_dir,
        ).unwrap();

        for i in 0..100 {
            let key = format!("k{}", i);
            let val = format!("v{}", i * i);
            let value: String = iter::repeat(val).take(i * i).collect::<Vec<_>>().concat();

            db.single_delete(&WriteOptions::default(), b"k5").unwrap();
            db.put(&WriteOptions::default(), key.as_bytes(), value.as_bytes())
                .unwrap();

            // make as many sst as possible
            assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
        }

        // Will be an Other add_user_key callback
        // assert!(db.delete_range_cf(WriteOptions::default_instance(), &db.default_column_family(),
        //                          b"k2", b"k6").is_ok());
        // assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());

        let props =
            db.get_properties_of_tables_in_range(&db.default_column_family(), &[b"k0".as_ref()..b"k9".as_ref()]);

        assert!(props.is_ok());
        let props = props.unwrap();

        assert!(props.len() > 0);
        for (file, prop) in props.iter() {
            assert!(file.ends_with(".sst"));
            assert!(prop.property_collectors_names().contains(
                "RustTablePropertiesCollectorFactory",
            ));

            let user_prop = prop.user_collected_properties();
            let diff_vals = user_prop.iter().collect::<Vec<_>>();
            assert_eq!(&user_prop["hello"], b"world");
            for (k, v) in user_prop.iter() {
                assert!(k.len() > 0); // has key
                assert!(v.len() > 0);
            }
        }

        let mut files = props.iter().map(|(file, _)| file).collect::<Vec<_>>();
        files.sort();
        files.dedup(); // assure files returned are all unique
        assert_eq!(files.len(), 100);
        let mut counters = props
            .iter()
            .map(|(_, props)| props.user_collected_properties()["test.counter"].to_vec())
            .collect::<Vec<_>>();
        counters.sort();
        counters.dedup(); // assure files returned are all unique
        assert_eq!(counters.len(), 100);
    }
}
