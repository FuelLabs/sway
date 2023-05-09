contract;

use std::block::{block_header_hash, height, timestamp, timestamp_of_block};
use block_test_abi::BlockTest;

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

    fn get_block_header_hash(h: u64) -> b256 {
        let res = block_header_hash(h);
        match res {
            Ok(h) => h,
            Err(e) => revert(0),
        }
    }
}
