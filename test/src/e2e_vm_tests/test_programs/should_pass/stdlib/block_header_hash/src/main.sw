script;

use std::{assert::assert, block::block_header_hash, constants::ZERO_B256};

fn main() -> bool {

    // Get the block header hash of the genesis block (guaranteed to exist)
    // Function handles cases where block, so if unwrap succeeds,
    // the header hash was succesfully retrieved
    let mut hash = block_header_hash(0).unwrap();

    // Try to get header hash of a block in the future
    // The function should return a BlockHashError
    let hash = block_header_hash(1_000_000_000);
    let did_error = match hash {
        Result::Ok(_) => false,
        Result::Err(BlockHashError) => true,
    };
    assert(did_error);

    true
}
