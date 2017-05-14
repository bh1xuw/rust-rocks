
/// Context information of a compaction run
#[repr(C)]
pub struct CompactionFilterContext {
    /// Does this compaction run include all data files
    is_full_compaction: bool,
    /// Is this compaction requested by the client (true),
    /// or is it occurring as an automatic compaction process
    is_manual_compaction: bool,
}


/// CompactionFilter allows an application to modify/delete a key-value at
/// the time of compaction.
pub struct CompactionFilter;


/// Each compaction will create a new CompactionFilter allowing the
/// application to know about different compactions
pub struct CompactionFilterFactory;
