//! Functionality for accessing block-related data.
library block;

use ::assert::assert;
use ::constants::ZERO_B256;
use ::result::Result;

enum BlockHashError {
    BlockHeightTooHigh: (),
}

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

/// Get the header hash of the block at height `block_height`
pub fn block_header_hash(block_height: u64) -> Result<b256, BlockHashError> {

    let mut header_hash = ZERO_B256;

    asm(r1: __addr_of(header_hash), r2: block_height) {
        bhsh r1 r2;
    };

    // `bhsh` returns b256(0) if the block is not found, so catch this and return an error
    if header_hash == ZERO_B256 {
        Result::Err(BlockHashError::BlockHeightTooHigh)
    } else {
        Result::Ok(header_hash)
    }
}

////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////

#[test()]
fn test_block_header_hash_ok() {

    // Get the block header hash of the previous block
    let mut hash = block_header_hash(height() - 1);
    assert(hash.is_ok());
}

#[test()]
fn test_block_header_hash_err() {

    // Try to get header hash of a block in the future
    // The function should return a BlockHashError
    let hash = block_header_hash(height() + 1);
    let correct_error = match hash {
        Result::Ok(_) => false,
        Result::Err(BlockHashError::BlockHeightTooHigh) => true,
    };

    assert(correct_error);
    
}
