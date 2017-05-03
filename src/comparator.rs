/// A Comparator object provides a total order across slices that are
/// used as keys in an sstable or a database.  A Comparator implementation
/// must be thread-safe since rocksdb may invoke its methods concurrently
/// from multiple threads.
pub struct Comparator;

impl Comparator {
    /// Return a builtin comparator that uses lexicographic byte-wise
    /// ordering.  The result remains the property of this module and
    /// must not be deleted.
    pub fn new_bytewise() -> Comparator {
        unimplemented!()
    }

    /// Return a builtin comparator that uses reverse lexicographic byte-wise
    /// ordering.
    pub fn new_reverse_bytewise() -> Comparator {
        unimplemented!()
    }

    /// Three-way comparison.  Returns value:
    ///   < 0 iff "a" < "b",
    ///   == 0 iff "a" == "b",
    ///   > 0 iff "a" > "b"
    pub fn compare(&self, a: &[u8], b: &[u8]) -> i32 {
        unimplemented!()
    }

    /// Compares two slices for equality. The following invariant should always
    /// hold (and is the default implementation):
    ///   Equal(a, b) iff Compare(a, b) == 0
    /// Overwrite only if equality comparisons can be done more efficiently than
    /// three-way comparisons.
    pub fn equal(&self, a: &[u8], b: &[u8]) -> bool {
        return self.compare(a, b) == 0;
    }
    /// The name of the comparator.  Used to check for comparator
    /// mismatches (i.e., a DB created with one comparator is
    /// accessed using a different comparator.
    ///
    /// The client of this package should switch to a new name whenever
    /// the comparator implementation changes in a way that will cause
    /// the relative ordering of any two keys to change.
    ///
    /// Names starting with "rocksdb." are reserved and should not be used
    /// by any clients of this package.
    pub fn name(&self) -> &'static str {
        unimplemented!()
    }
}
