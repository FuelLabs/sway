contract;

use std::block::{block_header_hash, chain_id, height, timestamp, timestamp_of_block};
use block_test_abi::BlockTest;

impl BlockTest for Contract {
    fn get_block_height() -> u32 {
        height()
    }

    fn get_timestamp() -> u64 {
        timestamp()
    }

    fn get_timestamp_of_block(block_height: u32) -> u64 {
        timestamp_of_block(block_height)
    }

    fn get_block_and_timestamp() -> (u32, u64) {
        (height(), timestamp())
    }

    fn get_block_header_hash(h: u32) -> b256 {
        let res = block_header_hash(h);
        match res {
            Ok(h) => h,
            Err(_) => revert(0),
        }
    }

    fn get_chain_id() -> u64 {
        chain_id()
    }
}
