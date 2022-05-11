contract;

use std::hash::{HashMethod, hash_u64};

abi MyContract {
    fn get_s256_hash_u64(value: u64) -> b256;
    fn get_k256_hash_u64(value: u64) -> b256;
}

impl MyContract for Contract {
    fn get_s256_hash_u64(value: u64) -> b256 {
        hash_u64(value, HashMethod::Sha256)
    }

    fn get_k256_hash_u64(value: u64) -> b256 {
        hash_u64(value, HashMethod::Keccak256)
    }
}
