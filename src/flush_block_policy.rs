//! Determine when to flush a block. TODO

use table::BlockBasedTableOptions;

// FlushBlockPolicy provides a configurable way to determine when to flush a
// block in the block based tables,
pub trait FlushBlockPolicy {
    // Keep track of the key/value sequences and return the boolean value to
    // determine if table builder should flush current data block.
    fn update(&mut self, key: &[u8], value: &[u8]) -> bool;
}

pub trait FlushBlockPolicyFactory {
    // Return the name of the flush block policy.
    fn name(&self) -> &str {
        "RustFlushBlockPolicyFactory\0"
    }

    // Return a new block flush policy that flushes data blocks by data size.
    // FlushBlockPolicy may need to access the metadata of the data block
    // builder to determine when to flush the blocks.
    //
    // Callers must delete the result after any database that is using the
    // result has been closed.
    fn new_flush_block_policy(&self, table_options: &BlockBasedTableOptions) -> Box<FlushBlockPolicy>;
}
