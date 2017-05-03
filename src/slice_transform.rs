/// A SliceTranform is a generic pluggable way of transforming one string
/// to another. Its primary use-case is in configuring rocksdb
/// to store prefix blooms by setting prefix_extractor in
/// ColumnFamilyOptions.
pub struct SliceTransform;
