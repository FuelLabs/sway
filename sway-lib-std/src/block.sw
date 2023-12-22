//! Functionality for accessing block-related data.
library;

use ::assert::assert;
use ::constants::ZERO_B256;
use ::result::Result::{*, self};
use ::logging::log;

/// Error type for when the block hash cannot be found.
enum BlockHashError {
    /// Error returned when the block hash cannot be found.
    BlockHeightTooHigh: (),
}

/// Get the current block height.
///
/// # Returns
///
/// * [u32] - The current block height.
///
/// # Examples
///
/// ```sway
/// use std::block::height;
///
/// fn foo() {
///     let current_height = height();
///     log(current_height);
/// }
/// ```
pub fn height() -> u32 {
    asm(height) {
        bhei height;
        height: u32
    }
}

/// Get the TAI64 timestamp of the current block.
///
/// # Additional Information
///
/// The TAI64 timestamp begins at 2^62 seconds before 1970, and ends at 2^62 seconds after 1970,
/// with a TAI second defined as the duration of 9192631770 periods of the radiation corresponding
/// to the transition between the two hyperfine levels of the ground state of the cesium atom.
///
/// # Returns
///
/// * [u64] - The TAI64 timestamp of the current block.
///
/// # Examples
///
/// ```sway
/// use std::block::timestamp;
///
/// fn foo() {
///     let current_timestamp = timestamp();
///     log(current_timestamp);
/// }
/// ```
pub fn timestamp() -> u64 {
    timestamp_of_block(height())
}

/// Get the TAI64 timestamp of a block at a given `block_height`.
///
/// # Additional Information
///
/// The TAI64 timestamp begins at 2^62 seconds before 1970, and ends at 2^62 seconds after 1970,
/// with a TAI second defined as the duration of 9192631770 periods of the radiation corresponding
/// to the transition between the two hyperfine levels of the ground state of the cesium atom.
///
/// # Arguments
///
/// * `block_height`: [u32] - The height of the block to get the timestamp of.
///
/// # Returns
///
/// * [u64] - The TAI64 timestamp of the block at `block_height`.
///
/// # Examples
///
/// ```sway
/// use std::block::timestamp_of_block;
///
/// fn foo() {
///     let timestamp_of_block_100 = timestamp_of_block(100u32);
///     log(timestamp_of_block_100);
/// }
/// ```
pub fn timestamp_of_block(block_height: u32) -> u64 {
    asm(timestamp, height: block_height) {
        time timestamp height;
        timestamp: u64
    }
}

/// Get the header hash of the block at height `block_height`
///
/// # Returns
///
/// * [Result<b256, BlockHashError>] - The header hash of the block at `block_height`, or a [BlockHashError] if the block is not found.
///
/// # Examples
///
/// ```sway
/// use std::block::block_header_hash;
///
/// fn foo() {
///     let header_hash_of_block_100 = block_header_hash(100u32);
///     log(header_hash_of_block_100);
/// }
/// ```
pub fn block_header_hash(block_height: u32) -> Result<b256, BlockHashError> {
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
fn test_block_header_hash_err_current_height() {
    // Get the header hash of the current block. Each time this test runs, the block height will be 1. calling BHSH with a height >= current height will fail.
    let mut hash = block_header_hash(height());
    let correct_error = match hash {
        Ok(_) => false,
        Err(BlockHashError::BlockHeightTooHigh) => true,
    };

    assert(correct_error);
}

#[test(should_revert)]
fn test_block_header_hash_err_future_height() {
    // Try to get header hash of a block in the future
    // The function should return a BlockHashError
    let hash = block_header_hash(height() + 1u32);
    let correct_error = match hash {
        Ok(_) => false,
        Err(BlockHashError::BlockHeightTooHigh) => true,
    };

    assert(correct_error);
}
