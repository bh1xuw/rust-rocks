//! A database can be configured with a custom `FilterPolicy` object.
//! This object is responsible for creating a small filter from a set
//! of keys.  These filters are stored in rocksdb and are consulted
//! automatically by rocksdb to decide whether or not to read some
//! information from disk. In many cases, a filter can cut down the
//! number of disk seeks form a handful to a single disk seek per
//! `DB::Get()` call.
//!
//! Most people will want to use the builtin bloom filter support (see
//! `NewBloomFilterPolicy()` below).

use rocks_sys as ll;

use to_raw::ToRaw;

pub struct FilterPolicy {
    raw: *mut ll::rocks_raw_filterpolicy_t,
}

impl ToRaw<ll::rocks_raw_filterpolicy_t> for FilterPolicy {
    fn raw(&self) -> *mut ll::rocks_raw_filterpolicy_t {
        self.raw
    }
}

impl Drop for FilterPolicy {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_raw_filterpolicy_destroy(self.raw);
        }
    }
}

impl FilterPolicy {
    // Return a new filter policy that uses a bloom filter with approximately
    // the specified number of bits per key.
    //
    // bits_per_key: bits per key in bloom filter. A good value for bits_per_key
    // is 10, which yields a filter with ~ 1% false positive rate.
    //
    // use_block_based_builder: use block based filter rather than full filter.
    // If you want to builder full filter, it needs to be set to false.
    //
    // Callers must delete the result after any database that is using the
    // result has been closed.
    //
    // Note: if you are using a custom comparator that ignores some parts
    // of the keys being compared, you must not use NewBloomFilterPolicy()
    // and must provide your own FilterPolicy that also ignores the
    // corresponding parts of the keys.  For example, if the comparator
    // ignores trailing spaces, it would be incorrect to use a
    // FilterPolicy (like NewBloomFilterPolicy) that does not ignore
    // trailing spaces in keys.
    pub fn new_bloom_filter(bits_per_key: i32, use_block_based_builder: bool) -> FilterPolicy {
        FilterPolicy {
            raw: unsafe { ll::rocks_raw_filterpolicy_new_bloomfilter(bits_per_key, use_block_based_builder as u8) },
        }
    }
}

// We add a new format of filter block called full filter block
// This new interface gives you more space of customization
//
// For the full filter block, you can plug in your version by implement
// the FilterBitsBuilder and FilterBitsReader
//
// There are two sets of interface in FilterPolicy
//
// Set 1: CreateFilter, KeyMayMatch: used for blockbased filter
//
// Set 2: GetFilterBitsBuilder, GetFilterBitsReader, they are used for
// full filter.
//
// Set 1 MUST be implemented correctly, Set 2 is optional
//
// RocksDB would first try using functions in Set 2. if they return nullptr,
// it would use Set 1 instead.
//
// You can choose filter type in NewBloomFilterPolicy
// pub trait FilterPolicy {
// Return the name of this policy.  Note that if the filter encoding
// changes in an incompatible way, the name returned by this method
// must be changed.  Otherwise, old incompatible filters may be
// passed to methods of this type.
// fn name(&self) -> &str {
// "RustFilterPolicy\0"
// }
//
// keys[0,n-1] contains a list of keys (potentially with duplicates)
// that are ordered according to the user supplied comparator.
// Append a filter that summarizes keys[0,n-1] to *dst.
//
// Warning: do not change the initial contents of *dst.  Instead,
// append the newly constructed filter to *dst.
//
// For Rust: must call dst.extend_from_slice() or dst.push()
// fn create_filter(&self, keys: &[&[u8]], dst: &mut Vec<u8>);
//
// "filter" contains the data appended by a preceding call to
// CreateFilter() on this class.  This method must return true if
// the key was in the list of keys passed to CreateFilter().
// This method may return true or false if the key was not on the
// list, but it should aim to return false with a high probability.
// fn key_may_match(&self, key: &[u8], filter: &[u8]) -> bool;
// }
//
