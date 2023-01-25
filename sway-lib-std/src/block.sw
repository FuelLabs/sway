//! Functionality for accessing block-related data.
library block;

/// Get the current block height.
pub fn height() -> u64 {
    asm(height) {
        bhei height;
        height: u64
    }
}

/// Get the timestamp of the current block.
pub fn timestamp() -> u64 {
    timestamp_of_block(height())
}

/// Get the timestamp of a block at a given `block_height`.
pub fn timestamp_of_block(block_height: u64) -> u64 {
    asm(timestamp, height: block_height) {
        time timestamp height;
        timestamp: u64
    }
}
