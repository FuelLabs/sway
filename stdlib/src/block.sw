library block;
//! Functionality for accessing block-related data.

/// Get the current block height
pub fn block_height() -> u64 {
    asm(height) {
        bhei height;
        height: u64
    }
}
