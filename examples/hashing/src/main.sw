script;

use std::hash::*;

impl Hash for Location {
    fn hash(self, ref mut state: Hasher) {
        match self {
            Location::Earth => {
                0_u8.hash(state);
            }
            Location::Mars => {
                1_u8.hash(state);
            }
        }
    }
}

impl Hash for Stats {
    fn hash(self, ref mut state: Hasher) {
        self.strength.hash(state);
        self.agility.hash(state);
    }
}

impl Hash for Person {
    fn hash(self, ref mut state: Hasher) {
        self.name.hash(state);
        self.age.hash(state);
        self.alive.hash(state);
        self.location.hash(state);
        self.stats.hash(state);
        self.some_tuple.hash(state);
        self.some_array.hash(state);
        self.some_b256.hash(state);
    }
}

const VALUE_A: b256 = 0x9280359a3b96819889d30614068715d634ad0cf9bba70c0f430a8c201138f79f;

enum Location {
    Earth: (),
    Mars: (),
}

struct Person {
    name: str,
    age: u64,
    alive: bool,
    location: Location,
    stats: Stats,
    some_tuple: (bool, u64),
    some_array: [u64; 2],
    some_b256: b256,
}

struct Stats {
    strength: u64,
    agility: u64,
}

fn main() {
    let zero = b256::min();
    // Use the generic sha256 to hash some integers
    let sha_hashed_u8 = sha256(u8::max());
    let sha_hashed_u16 = sha256(u16::max());
    let sha_hashed_u32 = sha256(u32::max());
    let sha_hashed_u64 = sha256(u64::max());

    // Or hash a b256
    let sha_hashed_b256 = sha256(VALUE_A);

    // You can hash booleans too
    let sha_hashed_bool = sha256(true);

    // Strings are not a problem either
    let sha_hashed_str = sha256("Fastest Modular Execution Layer!");

    // Tuples of any size work too
    let sha_hashed_tuple = sha256((true, 7));

    // As do arrays
    let sha_hashed_array = sha256([4, 5, 6]);

    // Enums work too
    let sha_hashed_enum = sha256(Location::Earth);

    // Complex structs are not a problem
    let sha_hashed_struct = sha256(Person {
        name: "John",
        age: 9000,
        alive: true,
        location: Location::Mars,
        stats: Stats {
            strength: 10,
            agility: 9,
        },
        some_tuple: (true, 8),
        some_array: [17, 76],
        some_b256: zero,
    });

    log(sha_hashed_u8);
    log(sha_hashed_u16);
    log(sha_hashed_u32);
    log(sha_hashed_u64);
    log(sha_hashed_b256);
    log(sha_hashed_bool);
    log(sha_hashed_str);
    log(sha_hashed_tuple);
    log(sha_hashed_array);
    log(sha_hashed_enum);
    log(sha_hashed_struct);

    // Use the generic keccak256 to hash some integers
    let keccak_hashed_u8 = keccak256(u8::max());
    let keccak_hashed_u16 = keccak256(u16::max());
    let keccak_hashed_u32 = keccak256(u32::max());
    let keccak_hashed_u64 = keccak256(u64::max());

    // Or hash a b256
    let keccak_hashed_b256 = keccak256(VALUE_A);

    // You can hash booleans too
    let keccak_hashed_bool = keccak256(true);

    // Strings are not a problem either
    let keccak_hashed_str = keccak256("Fastest Modular Execution Layer!");

    // Tuples of any size work too
    let keccak_hashed_tuple = keccak256((true, 7));

    // As do arrays
    let keccak_hashed_array = keccak256([4, 5, 6]);

    // Enums work too
    let keccak_hashed_enum = keccak256(Location::Earth);

    // Complex structs are not a problem
    let keccak_hashed_struct = keccak256(Person {
        name: "John",
        age: 9000,
        alive: true,
        location: Location::Mars,
        stats: Stats {
            strength: 10,
            agility: 9,
        },
        some_tuple: (true, 8),
        some_array: [17, 76],
        some_b256: zero,
    });

    log(keccak_hashed_u8);
    log(keccak_hashed_u16);
    log(keccak_hashed_u32);
    log(keccak_hashed_u64);
    log(keccak_hashed_b256);
    log(keccak_hashed_bool);
    log(keccak_hashed_str);
    log(keccak_hashed_tuple);
    log(keccak_hashed_array);
    log(keccak_hashed_enum);
    log(keccak_hashed_struct);
}
