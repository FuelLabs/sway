library;

use std::block::{block_header_hash, BlockHashError, height, timestamp, timestamp_of_block};

#[test]
fn block_height() {
    let h = height();
    assert(h >= 1u32);
}

#[test]
fn block_timestamp() {
    let time = timestamp();
    assert(time >= 1);
}

#[test]
fn block_timestamp_of_block() {
    let time = timestamp_of_block(height());
    assert(time >= 1);
}

#[test]
fn block_block_header_hash() {
    let result = block_header_hash(height());
    assert(result.is_ok());

    let hash = result.unwrap();
    assert(hash != b256::zero());
}

#[test(should_revert)]
fn revert_block_header_hash_err_current_height() {
    // Get the header hash of the current block. Each time this test runs, the block height will be 1. calling BHSH with a height >= current height will fail.
    let mut hash = block_header_hash(height());
    let correct_error = match hash {
        Ok(_) => false,
        Err(BlockHashError::BlockHeightTooHigh) => true,
    };

    assert(correct_error);
}

#[test(should_revert)]
fn revert_block_header_hash_err_future_height() {
    // Try to get header hash of a block in the future
    // The function should return a BlockHashError
    let hash = block_header_hash(height() + 1u32);
    let correct_error = match hash {
        Ok(_) => false,
        Err(BlockHashError::BlockHeightTooHigh) => true,
    };

    assert(correct_error);
}
