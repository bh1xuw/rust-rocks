//! The Merge Operator
//!
//! Essentially, a `MergeOperator` specifies the SEMANTICS of a merge, which only
//! client knows. It could be numeric addition, list append, string
//! concatenation, edit data structure, ... , anything.
//! The library, on the other hand, is concerned with the exercise of this
//! interface, at the right time (during get, iteration, compaction...)
//!
//! To use merge, the client needs to provide an object implementing one of
//! the following interfaces:
//!
//! * [`AssociativeMergeOperator`] - for most simple semantics (always take
//!   two values, and merge them into one value, which is then put back
//!   into rocksdb); numeric addition and string concatenation are examples;
//!
//! * [`MergeOperator`] - the generic class for all the more abstract / complex
//!   operations; one method (FullMergeV2) to merge a Put/Delete value with a
//!   merge operand; and another method (PartialMerge) that merges multiple
//!   operands together. this is especially useful if your key values have
//!   complex structures but you would still like to support client-specific
//!   incremental updates.
//!
//! [`AssociativeMergeOperator`] is simpler to implement. [`MergeOperator`] is simply
//! more powerful.
//!
//! Refer to rocksdb-merge wiki for more details and example implementations.
//!
//! [`AssociativeMergeOperator`]: ../../rocks/merge_operator/trait.AssociativeMergeOperator.html
//! [`MergeOperator`]: ../../rocks/merge_operator/trait.MergeOperator.html

use std::ptr;
use std::mem;
use std::marker::PhantomData;
use std::slice;

use rocks_sys as ll;

use env::Logger;

// really unsafe.
// &&[u8] is almost the same as &Slice

#[repr(C)]
pub struct MergeOperationInput<'a> {
    /// The key associated with the merge operation.
    pub key: &'a &'a [u8],
    /// The existing value of the current key, nullptr means that the
    /// value dont exist.
    pub existing_value: Option<&'a &'a [u8]>,
    operand_list: *mut (),
    logger: *mut (),
    _marker: PhantomData<&'a ()>,
}

impl<'a> MergeOperationInput<'a> {
    /// A list of operands to apply.
    pub fn operands(&self) -> &[&[u8]] {
        unsafe {
            slice::from_raw_parts(
                ll::cxx_vector_slice_nth(self.operand_list as *const _, 0) as *const _,
                ll::cxx_vector_slice_size(self.operand_list as *const _),
            )
        }
    }

    /// Logger could be used by client to log any errors that happen during
    /// the merge operation.
    pub fn logger(&self) -> &Logger {
        unimplemented!()
    }
}


#[repr(C)]
pub struct MergeOperationOutput<'a> {
    /// Client is responsible for filling the merge result here.
    new_value: *mut (),
    /// If the merge result is one of the existing operands (or existing_value),
    /// client can set this field to the operand (or existing_value) instead of
    /// using new_value.
    existing_operand: *mut &'a [u8],
}


impl<'a> MergeOperationOutput<'a> {
    /// Client is responsible for filling the merge result here.
    pub fn assign(&mut self, new_value: &[u8]) {
        unsafe {
            ll::cxx_string_assign(self.new_value as *mut _, new_value.as_ptr() as *const _, new_value.len());
        }
    }

    /// If the merge result is one of the existing operands (or existing_value),
    /// client can set this field to the operand (or existing_value) instead of
    /// using new_value.
    // FIXME: not works
    pub fn assign_existing_operand(&mut self, old_value: &&[u8]) {
        // :( transmute for disable lifetime checker
        self.existing_operand = old_value as *const &[u8] as *mut &'a [u8];
    }
}


/// `MergeOperator` - the generic class for all the more abstract / complex
/// operations; one method (FullMergeV2) to merge a Put/Delete value with a
/// merge operand; and another method (PartialMerge) that merges multiple
/// operands together. this is especially useful if your key values have
/// complex structures but you would still like to support client-specific
/// incremental updates.
pub trait MergeOperator {
    /// Gives the client a way to express the read -> modify -> write semantics
    ///
    /// # Arguments
    ///
    /// * `key` - (IN) The key that's associated with this merge operation.
    ///   Client could multiplex the merge operator based on it
    ///   if the key space is partitioned and different subspaces
    ///   refer to different types of data which have different
    ///   merge operation semantics
    /// * `existing` - (IN) null indicates that the key does not exist before this op
    /// * `operand_list` - (IN) the sequence of merge operations to apply, front() first.
    /// * `new_value` - (OUT) Client is responsible for filling the merge result here.
    ///   The string that new_value is pointing to will be empty.
    /// * `logger` - (IN) Client could use this to log errors during merge.
    ///
    /// Return true on success.
    ///
    /// All values passed in will be client-specific values. So if this method
    /// returns false, it is because client specified bad data or there was
    /// internal corruption. This will be treated as an error by the library.
    ///
    /// Also make use of the *logger for error messages.
    // use FullMergeV2
    // https://www.facebook.com/groups/rocksdb.dev/permalink/1023193664445814/
    fn full_merge(&self, merge_in: &MergeOperationInput, merge_out: &mut MergeOperationOutput) -> bool {
        false
    }

    // TODO: PartialMerge

    /// The name of the MergeOperator. Used to check for MergeOperator
    /// mismatches (i.e., a DB created with one MergeOperator is
    /// accessed using a different MergeOperator)
    ///
    /// TODO: the name is currently not stored persistently and thus
    ///       no checking is enforced. Client is responsible for providing
    ///       consistent MergeOperator between DB opens.
    // FIXME: \0 is required
    fn name(&self) -> &str {
        "RustMergeOperator\0"
    }
}

/// `AssociativeMergeOperator` - for most simple semantics (always take
/// two values, and merge them into one value, which is then put back
/// into rocksdb); numeric addition and string concatenation are examples;
pub trait AssociativeMergeOperator {
    /// Gives the client a way to express the read -> modify -> write semantics
    ///
    /// # Arguments
    ///
    /// * `key` - (IN) The key that's associated with this merge operation.
    /// * `existing_value` - (IN) null indicates the key does not exist before this op
    /// * `value` - (IN) the value to update/merge the existing_value with
    /// * `new_value` - (OUT) Client is responsible for filling the merge result
    ///   here. The string that new_value is pointing to will be empty.
    /// * `logger` - (IN) Client could use this to log errors during merge.
    ///
    /// Return true on success.
    ///
    /// All values passed in will be client-specific values. So if this method
    /// returns false, it is because client specified bad data or there was
    /// internal corruption. The client should assume that this will be treated
    /// as an error by the library.
    fn merge(&self, key: &[u8], existing_value: Option<&[u8]>, value: &[u8], logger: &Logger) -> Option<Vec<u8>>;


    /// The name of the MergeOperator. Used to check for MergeOperator
    /// mismatches (i.e., a DB created with one MergeOperator is
    /// accessed using a different MergeOperator)
    ///
    /// TODO: the name is currently not stored persistently and thus
    ///       no checking is enforced. Client is responsible for providing
    ///       consistent MergeOperator between DB opens.
    // FIXME: \0 is required
    fn name(&self) -> &str {
        "RustAssociativeMergeOperator\0"
    }
}


// call rust fn in C
#[doc(hidden)]
pub mod c {
    use super::*;

    #[no_mangle]
    pub extern "C" fn rust_merge_operator_call_full_merge_v2(
        op: *mut (),
        merge_in: *const MergeOperationInput,
        merge_out: *mut MergeOperationOutput,
    ) -> i32 {
        assert!(!op.is_null());
        unsafe {
            let operator = op as *mut Box<MergeOperator>;
            let m_in: &MergeOperationInput = &*(merge_in as *const MergeOperationInput);
            let m_out: &mut MergeOperationOutput = &mut *(merge_out as *mut MergeOperationOutput);
            let ret = (*operator).full_merge(m_in, m_out);
            ret as i32
        }
    }

    #[no_mangle]
    pub extern "C" fn rust_merge_operator_drop(op: *mut ()) {
        assert!(!op.is_null());
        unsafe {
            let operator = op as *mut Box<MergeOperator>;
            Box::from_raw(operator);
        }
    }

    #[no_mangle]
    pub extern "C" fn rust_associative_merge_operator_call(
        op: *mut (),
        key: &&[u8],
        existing_value: Option<&&[u8]>,
        value: &&[u8],
        new_value: *mut *const u8,
        new_value_len: *mut usize,
        logger: &Logger,
    ) -> i32 {
        // FIXME: this is very dangerous and unsafe play.
        assert!(!op.is_null());
        unsafe {
            let operator = op as *mut Box<AssociativeMergeOperator>;
            let nval = (*operator).merge(*key, existing_value.map(|&s| s), *value, logger);
            if let Some(val) = nval {
                *new_value_len = val.len();
                *new_value = val.as_ptr();
                // NOTE: this val is dropped in C by `rust_drop_vec_u8`
                mem::forget(val);
                true as _
            } else {
                false as _
            }
        }
    }

    // trait object is also 2 pointer size
    #[no_mangle]
    pub extern "C" fn rust_associative_merge_operator_name(op: *mut ()) -> *const u8 {
        assert!(!op.is_null());
        unsafe {
            let operator = op as *mut Box<AssociativeMergeOperator>;
            (*operator).name().as_bytes().as_ptr()
        }
    }


    // trait object is also 2 pointer size
    #[no_mangle]
    pub extern "C" fn rust_merge_operator_name(op: *mut ()) -> *const u8 {
        assert!(!op.is_null());
        unsafe {
            let operator = op as *mut Box<MergeOperator>;
            (*operator).name().as_bytes().as_ptr()
        }
    }

    #[no_mangle]
    pub extern "C" fn rust_drop_vec_u8(base: *mut u8, len: usize) {
        unsafe {
            // FIXME: is capacity same as length is ok for a Drop?
            Vec::from_raw_parts(base, len, len);
        }
    }

    #[no_mangle]
    pub extern "C" fn rust_associative_merge_operator_drop(op: *mut ()) {
        assert!(!op.is_null());
        unsafe {
            let operator = op as *mut Box<AssociativeMergeOperator>;
            Box::from_raw(operator);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::rocksdb::*;

    pub struct MyAssocMergeOp;

    impl AssociativeMergeOperator for MyAssocMergeOp {
        fn merge(&self, key: &[u8], existing_value: Option<&[u8]>, value: &[u8], logger: &Logger) -> Option<Vec<u8>> {
            Some(b"welcome to china".to_vec())
        }
    }

    #[test]
    fn it_works() {
        let op: Box<AssociativeMergeOperator> = Box::new(MyAssocMergeOp);
    }

    #[test]
    fn assoc_merge() {
        use tempdir::TempDir;
        let tmp_dir = TempDir::new_in(".", "rocks").unwrap();

        pub struct MyAssocMergeOp;

        impl AssociativeMergeOperator for MyAssocMergeOp {
            fn merge(
                &self,
                key: &[u8],
                existing_value: Option<&[u8]>,
                value: &[u8],
                logger: &Logger,
            ) -> Option<Vec<u8>> {

                let mut ret: Vec<u8> = existing_value.map(|s| s.into()).unwrap_or(b"HEAD".to_vec());
                ret.push(b'|');
                ret.extend_from_slice(value);
                Some(ret)
            }
        }

        let db = DB::open(
            Options::default()
                .map_db_options(|db| db.create_if_missing(true))
                .map_cf_options(|cf| cf.associative_merge_operator(Box::new(MyAssocMergeOp))),
            tmp_dir,
        ).unwrap();

        let ret = db.merge(&WriteOptions::default(), b"name", b"value");
        let ret = db.merge(&WriteOptions::default(), b"name", b"value2");
        let ret = db.merge(&WriteOptions::default(), b"name", b"value3");
        let ret = db.merge(&WriteOptions::default(), b"gender", b"male");
        let ret = db.merge(&WriteOptions::default(), b"name", b"value4");
        let ret = db.merge(&WriteOptions::default(), b"name", b"value");

        let ret = db.get(&ReadOptions::default(), b"name");
        assert_eq!(String::from_utf8_lossy(ret.unwrap().as_ref()), "HEAD|value|value2|value3|value4|value");
    }

    #[test]
    fn merge_assign_concat_operands() {
        use tempdir::TempDir;
        use merge_operator::{MergeOperationInput, MergeOperationOutput};

        let tmp_dir = TempDir::new_in(".", "rocks").unwrap();

        pub struct MyMergeOp;

        impl MergeOperator for MyMergeOp {
            fn full_merge(&self, merge_in: &MergeOperationInput, merge_out: &mut MergeOperationOutput) -> bool {
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

        let db = DB::open(
            Options::default()
                .map_db_options(|db| db.create_if_missing(true))
                .map_cf_options(|cf| cf.merge_operator(Box::new(MyMergeOp))),
            tmp_dir,
        ).unwrap();

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
    fn merge_assign_existing_operand() {
        use merge_operator::{MergeOperationInput, MergeOperationOutput};

        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();

        pub struct MyMergeOp;

        impl MergeOperator for MyMergeOp {
            fn full_merge(&self, merge_in: &MergeOperationInput, merge_out: &mut MergeOperationOutput) -> bool {
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

        let db = DB::open(
            Options::default()
                .map_db_options(|db| db.create_if_missing(true))
                .map_cf_options(|cf| cf.merge_operator(Box::new(MyMergeOp))),
            &tmp_dir,
        ).unwrap();

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
}
