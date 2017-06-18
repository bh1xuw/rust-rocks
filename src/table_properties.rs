//! TableProperties contains a bunch of read-only properties of its associated
//! table.

use std::os::raw::c_void;
use std::u32;
use std::slice;
use std::str;
use std::fmt;
use std::mem;
use std::marker::PhantomData;

use rocks_sys as ll;

use types::SequenceNumber;
use to_raw::{ToRaw, FromRaw};

pub const UNKNOWN_COLUMN_FAMILY_ID: u32 = u32::MAX;

/// A collections of table properties objects, where
///  key: is the table's file name.
///  value: the table properties object of the given table.
pub struct TablePropertiesCollection {
    // std::unordered_map<std::string, std::shared_ptr<const TableProperties>>
    raw: *mut ll::rocks_table_props_collection_t,
}

impl FromRaw<ll::rocks_table_props_collection_t> for TablePropertiesCollection {
    unsafe fn from_ll(raw: *mut ll::rocks_table_props_collection_t) -> Self {
        TablePropertiesCollection {
            raw: raw
        }
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
        unsafe {
            ll::rocks_table_props_collection_size(self.raw) as usize
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter<'a>(&'a self) -> TablePropertiesCollectionIter<'a> {
        TablePropertiesCollectionIter {
            raw: unsafe { ll::rocks_table_props_collection_iter_create(self.raw) },
            size: self.len(),
            at_end: self.is_empty(),
            _marker: PhantomData,
        }
    }
}

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
    type Item = (String, TableProperties<'a>);

    fn next(&mut self) -> Option<(String, TableProperties<'a>)> {
        if self.raw.is_null() || self.at_end {
            None
        } else {
            let mut key = String::new();
            unsafe {
                ll::rocks_table_props_collection_iter_key(self.raw,
                                                          &mut key as *mut String as *mut c_void);
                let prop = TableProperties::from_ll(ll::rocks_table_props_collection_iter_value(self.raw));
                self.at_end = ll::rocks_table_props_collection_iter_next(self.raw) == 0;
                // FIXME: can't use &str here, since each time iterator->first will be reused
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
/// Rust: wraps, and exposes a `{String => Vec<u8>}` map
#[repr(C)]
#[derive(Debug)]
pub struct UserCollectedProperties {
    // *std::map<std::string, std::string>
    raw: *mut c_void,
}

impl ToRaw<ll::rocks_user_collected_props_t> for UserCollectedProperties {
    fn raw(&self) -> *mut ll::rocks_user_collected_props_t {
        unsafe {
            mem::transmute(self as *const UserCollectedProperties as *mut c_void)
        }
    }
}

impl UserCollectedProperties {
    pub fn insert(&mut self, k: &str, v: &[u8]) {
        unimplemented!()
    }

    pub fn len(&self) -> usize {
        unsafe {
            ll::rocks_user_collected_props_size(self.raw()) as usize
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter<'a>(&'a self) -> UserCollectedPropertiesIter<'a> {
        UserCollectedPropertiesIter {
            raw: unsafe { ll::rocks_user_collected_props_iter_create(self.raw()) },
            size: self.len(),
            at_end: self.is_empty(),
            _marker: PhantomData,
        }
    }
}

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
    // FIXME: is {String => Vec<u8>} right?
    type Item = (String, Vec<u8>);

    fn next(&mut self) -> Option<(String, Vec<u8>)> {
        if self.raw.is_null() || self.at_end {
            None
        } else {
            let mut key = String::new();
            let mut value = Vec::new();
            unsafe {
                ll::rocks_user_collected_props_iter_key(self.raw,
                                                        &mut key as *mut String as *mut c_void);
                ll::rocks_user_collected_props_iter_value(self.raw,
                                                        &mut value as *mut Vec<u8> as *mut c_void);
                self.at_end = ll::rocks_user_collected_props_iter_next(self.raw) == 0;
            }
            Some((key, value))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.size, Some(self.size))
    }
}





#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum EntryType {
    EntryPut,
    EntryDelete,
    EntrySingleDelete,
    EntryMerge,
    EntryOther,
}



/// `TablePropertiesCollector` provides the mechanism for users to collect
/// their own properties that they are interested in. This class is essentially
/// a collection of callback functions that will be invoked during table
/// building. It is construced with TablePropertiesCollectorFactory. The methods
/// don't need to be thread-safe, as we will create exactly one
/// TablePropertiesCollector object per table and then call it sequentially
pub trait TablePropertiesCollector {
    /// AddUserKey() will be called when a new key/value pair is inserted into the
    /// table.
    /// 
    /// @params key    the user key that is inserted into the table.
    /// @params value  the value that is inserted into the table.
    fn add_user_key(&mut self, key: &[u8],  value: &[u8],
                    type_: EntryType, seq: SequenceNumber,
                    file_size: u64);

    /// Finish() will be called when a table has already been built and is ready
    /// for writing the properties block.
    /// @params properties  User will add their collected statistics to
    /// `properties`.
    fn finish(&mut self, properties: &mut UserCollectedProperties);

    /// The name of the properties collector can be used for debugging purpose.
    fn name(&self) -> &str {
        "RustTablePropertiesCollector\0"
    }

    // fn need_compact(&self) -> bool
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Context {
    column_family_id: u32,
}

// Constructs TablePropertiesCollector. Internals create a new
// TablePropertiesCollector for each new table
pub trait TablePropertiesCollectorFactory {
    fn create_table_properties_collector(&mut self, context: Context) -> Box<TablePropertiesCollector>;

    fn name(&self) -> &str {
        "RustTablePropertiesCollectorFactory\0"
    }
}


/// TableProperties contains a bunch of read-only properties of its associated
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
        write!(f, "TableProperties({:?})", self.to_string())
    }
}



impl<'a> TableProperties<'a> {
    /// the total size of all data blocks.
    pub fn data_size(&self) -> u64 {
        unsafe {
            ll::rocks_table_props_get_data_size(self.raw)
        }
    }
    /// the size of index block.
    pub fn index_size(&self) -> u64 {
        unsafe {
            ll::rocks_table_props_get_index_size(self.raw)
        }
    }
    /// the size of filter block.
    pub fn filter_size(&self) -> u64 {
        unsafe {
            ll::rocks_table_props_get_filter_size(self.raw)
        }
    }
    /// total raw key size
    pub fn raw_key_size(&self) -> u64 {
        unsafe {
            ll::rocks_table_props_get_raw_key_size(self.raw)
        }
    }
    /// total raw value size
    pub fn raw_value_size(&self) -> u64 {
        unsafe {
            ll::rocks_table_props_get_raw_value_size(self.raw)
        }
    }
    /// the number of blocks in this table
    pub fn num_data_blocks(&self) -> u64 {
        unsafe {
            ll::rocks_table_props_get_num_data_blocks(self.raw)
        }
    }
    /// the number of entries in this table
    pub fn num_entries(&self) -> u64 {
        unsafe {
            ll::rocks_table_props_get_num_entries(self.raw)
        }
    }
    /// format version, reserved for backward compatibility
    pub fn format_version(&self) -> u64 {
        unsafe {
            ll::rocks_table_props_get_format_version(self.raw)
        }
    }
    /// If 0, key is variable length. Otherwise number of bytes for each key.
    pub fn fixed_key_len(&self) -> u64 {
        unsafe {
            ll::rocks_table_props_get_format_version(self.raw)
        }
    }
    /// ID of column family for this SST file, corresponding to the CF identified
    /// by column_family_name.
    pub fn column_family_id(&self) -> u32 {
        unsafe {
            ll::rocks_table_props_get_column_family_id(self.raw)
        }
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
            mem::transmute(raw_ptr)
        }
    }

    pub fn readable_properties(&self) -> &UserCollectedProperties {
        unsafe {
            let raw_ptr = ll::rocks_table_props_get_readable_properties(self.raw);
            mem::transmute(raw_ptr)
        }
    }
}

