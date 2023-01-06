library block;

use ::assert::assert;
use ::constants::ZERO_B256;
use ::result::Result;

enum BlockHashError {
    BlockHeightTooHigh: (),
}

//! Functionality for accessing block-related data.
/// Get the current block height
pub fn height() -> u64 {
    asm(height) {
        bhei height;
        height: u64
    }
}

/// Get the timestamp of the current block
pub fn timestamp() -> u64 {
    timestamp_of_block(height())
}

/// Get the timestamp of block at height `block_height`
pub fn timestamp_of_block(block_height: u64) -> u64 {
    asm(timestamp, height: block_height) {
        time timestamp height;
        timestamp: u64
    }
}

/// Get the header hash of the block at height `block_height`
pub fn block_header_hash(block_height: u64) -> Result<b256, BlockHashError> {
    let header_hash = asm(r1, r2: block_height) {
        bhsh r1 r2;
        r1: b256
    };

    // `bhsh` returns b256(0) if the block is not found, so catch this and return explicit error
    match header_hash {
        ZERO_B256 => Result::Err(BlockHashError::BlockHeightTooHigh),
        _ => Result::Ok(header_hash),
    }
}

////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////

#[test()]
fn test_block_header_hash() {

// Get the block header hash of the genesis block (guaranteed to exist)
    let mut hash = block_header_hash(0);
    assert(hash.is_ok());

    // Try to get header hash of a block in the future
    // The function should return a BlockHashError
    let hash = block_header_hash(1_000_000_000);
    let did_error = match hash {
        Result::Ok(_) => false,
        Result::Err(BlockHashError) => true,
    };

    assert(did_error);

}
