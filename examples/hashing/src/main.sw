script;

use std::{
    chain::log_b256,
    constants::ZERO,
    core::num::*,
    hash::{HashMethod, hash_pair, hash_u64, hash_value, sha256}
};

const VALUE_A = 0x9280359a3b96819889d30614068715d634ad0cf9bba70c0f430a8c201138f79f;

enum Location {
    Earth: (),
    Mars: (),
}

struct Person {
    name: str[4],
    age: u64,
    alive: bool,
    location: Location,
    stats: Stats,
    some_tuple: (bool,
    u64), some_array: [u64;
    2],
    some_b256: b256,
}

struct Stats {
    strength: u64,
    agility: u64,
}

fn main() {
    // Use the generic sha256 to hash some integers
    let sha_hashed_u8 = sha256(~u8::max());
    let sha_hashed_u16 = sha256(~u16::max());
    let sha_hashed_u32 = sha256(~u32::max());
    let sha_hashed_u64 = sha256(~u64::max());

    // Or hash a b256
    let sha_hashed_b256 = sha256(VALUE_A);

    // You can hash booleans too
    let sha_hashed_bool = sha256(true);

    // Strings are not a problem either
    let sha_hashed_str = sha256( "Fastest Modular Execution Layer!");

    // Tuples of any size work too
    let sha_hashed_tuple = sha256((true, 7));

    // As do arrays
    let sha_hashed_array = sha256([4, 5, 6]);

    // Enums work too
    let sha_hashed_enum = sha256(Location::Earth);

    // Complex structs are not a problem
    let sha_hashed_struct = sha256(Person {
        name: "John", age: 9000, alive: true, location: Location::Mars, stats: Stats {
            strength: 10, agility: 9
        },
        some_tuple: (true, 8), some_array: [17, 76], some_b256: ZERO
    });

    log_b256(sha_hashed_u8);
    log_b256(sha_hashed_u16);
    log_b256(sha_hashed_u32);
    log_b256(sha_hashed_u64);
    log_b256(sha_hashed_b256);
    log_b256(sha_hashed_bool);
    log_b256(sha_hashed_str);
    log_b256(sha_hashed_tuple);
    log_b256(sha_hashed_array);
    log_b256(sha_hashed_enum);
    log_b256(sha_hashed_struct);

    // Hash a single u64 value.
    let hashed_u64 = hash_u64(100, HashMethod::Sha256);

    // Hash a single b256 value.
    let hashed_b256 = hash_value(hashed_u64, HashMethod::Keccak256);

    // Hash two b256 values.
    let hashed_pair = hash_pair(hashed_b256, VALUE_A, HashMethod::Sha256);

    log_b256(hashed_pair);
}
