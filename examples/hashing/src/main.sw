script;

use std::chain::log_b256;
use std::core::num::*;
use std::hash::{HashMethod, hash_pair, hash_u64, hash_value, sha256};

const VALUE_A = 0x9280359a3b96819889d30614068715d634ad0cf9bba70c0f430a8c201138f79f;

enum Location {
    Earth: ()
}

struct Person {
    name: str[4],
    age: u64,
    alive: bool
}

fn main() {
    // Use the generic sha256 to hash some values
    let sha_hashed_u8 = sha256(~u8::MAX());
    log_b256(sha_hashed_u8);

    let sha_hashed_u16 = sha256(~u16::MAX());
    log_b256(sha_hashed_u16);

    let sha_hashed_u32 = sha256(~u32::MAX());
    log_b256(sha_hashed_u32);

    let sha_hashed_u64 = sha256(~u64::MAX());
    log_b256(sha_hashed_u64);

    let sha_hashed_b256 = sha256(VALUE_A);
    log_b256(sha_hashed_b256);

    let sha_hashed_bool = sha256(true);
    log_b256(sha_hashed_bool);

    let sha_hashed_str = sha256("Fastest Modular Execution Layer!");
    log_b256(sha_hashed_str);

    let sha_hashed_tuple = sha256((true, 7));
    log_b256(sha_hashed_tuple);

    let sha_hashed_array = sha256([4, 5, 6]);
    log_b256(sha_hashed_array);

    let sha_hashed_enum = sha256(Location::Earth);
    log_b256(sha_hashed_enum);

    let sha_hashed_struct = sha256(Person { name: "John", age: 9000, alive: true });
    log_b256(sha_hashed_struct);

    // Hash a single u64 value.
    let hashed_u64 = hash_u64(100, HashMethod::Sha256);

    // Hash a single b256 value.
    let hashed_b256 = hash_value(hashed_u64, HashMethod::Keccak256);

    // Hash two b256 values.
    let hashed_pair = hash_pair(hashed_b256, VALUE_A, HashMethod::Sha256);

    log_b256(hashed_pair);
}
