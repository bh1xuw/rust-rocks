//! The Merge Operator
//!
//! Essentially, a MergeOperator specifies the SEMANTICS of a merge, which only
//! client knows. It could be numeric addition, list append, string
//! concatenation, edit data structure, ... , anything.
//! The library, on the other hand, is concerned with the exercise of this
//! interface, at the right time (during get, iteration, compaction...)
//!
//! To use merge, the client needs to provide an object implementing one of
//! the following interfaces:
//!  a) AssociativeMergeOperator - for most simple semantics (always take
//!    two values, and merge them into one value, which is then put back
//!    into rocksdb); numeric addition and string concatenation are examples;
//!
//!  b) MergeOperator - the generic class for all the more abstract / complex
//!    operations; one method (FullMergeV2) to merge a Put/Delete value with a
//!    merge operand; and another method (PartialMerge) that merges multiple
//!    operands together. this is especially useful if your key values have
//!    complex structures but you would still like to support client-specific
//!    incremental updates.
//!
//! AssociativeMergeOperator is simpler to implement. MergeOperator is simply
//! more powerful.
//!
//! Refer to rocksdb-merge wiki for more details and example implementations.
//!

use std::ptr;
use std::mem;

use env::Logger;

/// MergeOperator - the generic class for all the more abstract / complex
/// operations; one method (FullMergeV2) to merge a Put/Delete value with a
/// merge operand; and another method (PartialMerge) that merges multiple
/// operands together. this is especially useful if your key values have
/// complex structures but you would still like to support client-specific
/// incremental updates.
pub struct MergeOperator;


/*
pub trait MergeOperator {
    fn full_merge(key: &[u8], existing_value: Option<&[u8]>,
                  operand_list: &[&str], logger: &Logger) -> Option<Vec<u8>> {
        unimplemented!()
    }



}*/


/// The simpler, associative merge operator.
pub trait AssociativeMergeOperator {
    /// Gives the client a way to express the read -> modify -> write semantics
    /// key:           (IN) The key that's associated with this merge operation.
    /// existing_value:(IN) null indicates the key does not exist before this op
    /// value:         (IN) the value to update/merge the existing_value with
    /// new_value:    (OUT) Client is responsible for filling the merge result
    /// here. The string that new_value is pointing to will be empty.
    /// logger:        (IN) Client could use this to log errors during merge.
    ///
    /// Return true on success.
    /// All values passed in will be client-specific values. So if this method
    /// returns false, it is because client specified bad data or there was
    /// internal corruption. The client should assume that this will be treated
    /// as an error by the library.
    fn merge(&self, key: &[u8], existing_value: Option<&[u8]>,
             value: &[u8], logger: &Logger) -> Option<Vec<u8>>;


    /// The name of the MergeOperator. Used to check for MergeOperator
    /// mismatches (i.e., a DB created with one MergeOperator is
    /// accessed using a different MergeOperator)
    /// TODO: the name is currently not stored persistently and thus
    ///       no checking is enforced. Client is responsible for providing
    ///       consistent MergeOperator between DB opens.
    /// FIXME: \0 is required
    fn name(&self) -> &'static str {
        "AssociativeMergeOperator\0"
    }
}

// call rust fn in C
#[no_mangle]
pub extern "C" fn rust_associative_merge_operator_call(op: *mut (), key: &&[u8],
                                                   existing_value: Option<&&[u8]>,
                                                   value: &&[u8],
                                                   new_value: *mut *const u8, new_value_len: *mut usize,
                                                   logger: &Logger) -> i32 {
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



#[no_mangle]
pub extern "C" fn rust_drop_vec_u8(base: *mut u8, len: usize) {
    unsafe {
        // FIXME: is capacity same as length is ok for a Drop?
        Vec::from_raw_parts(base, len, len);
    }
}


#[no_mangle]
pub extern "C" fn rust_hello_world() {
    println!("Hello World! from rust");
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct MyAssocMergeOp;

    impl AssociativeMergeOperator for MyAssocMergeOp {
        fn merge(&self, key: &[u8], existing_value: Option<&[u8]>,
                 value: &[u8], logger: &Logger) -> Option<Vec<u8>> {
            Some(b"welcome to china".to_vec())
        }
    }

    #[test]
    fn it_works() {
        let op: Box<AssociativeMergeOperator> = Box::new(MyAssocMergeOp);
    }
}
