//! Functionality for accessing block-related data.
library;

use ::assert::assert;
use ::constants::ZERO_B256;
use ::result::Result::{self, *};
use ::logging::log;

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

/// Get the TAI64 timestamp of the current block.
pub fn timestamp() -> u64 {
    timestamp_of_block(height())
}

/// Get the TAI64 timestamp of a block at a given `block_height`.
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
        Err(BlockHashError::BlockHeightTooHigh)
    } else {
        Ok(header_hash)
    }
}

////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////
 
#[test(should_revert)]
fn block_test_header_hash_err_current_height() {
    // Get the header hash of the current block. Each time this test runs, the block height will be 1. calling BHSH with a height >= current height will fail.
    let mut hash = block_header_hash(height());
    let correct_error = match hash {
        Ok(_) => false,
        Err(BlockHashError::BlockHeightTooHigh) => true,
    };

    assert(correct_error);
}

#[test(should_revert)]
fn block_test_header_hash_err_future_height() {

    // Try to get header hash of a block in the future
    // The function should return a BlockHashError
    let hash = block_header_hash(height() + 1);
    let correct_error = match hash {
        Ok(_) => false,
        Err(BlockHashError::BlockHeightTooHigh) => true,
    };

    assert(correct_error);
    
}
