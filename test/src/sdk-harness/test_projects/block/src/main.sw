contract;

use std::block::*;
use block_test_abi::*;

impl BlockTest for Contract {
    fn get_block_height() -> u64 {
        height()
    }

    fn get_timestamp() -> u64 {
        timestamp()
    }

    fn get_timestamp_of_block(block_height: u64) -> u64 {
        timestamp_of_block(block_height)
    }

    fn get_block_and_timestamp() -> (u64, u64) {
        (height(), timestamp())
    }
}
