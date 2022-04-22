script;

use std::chain::log_b256;
use std::hash::{
    HashMethod,
    hash_pair,
    hash_u64,
    hash_value
};

const VALUE_A = 0x9280359a3b96819889d30614068715d634ad0cf9bba70c0f430a8c201138f79f;

fn main() {
    // Hash a single u64 value.
    let hashed_u64 = hash_u64(100, HashMethod::Sha256);

    // Hash a single b256 value.
    let hashed_b256 = hash_value(hashed_u64, HashMethod::Keccak256);

    // Hash two b256 values.
    let hashed_pair = hash_pair(hashed_b256, VALUE_A, HashMethod::Sha256);

    log_b256(hashed_pair);
}
