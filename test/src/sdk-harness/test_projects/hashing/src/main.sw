contract;

use std::{
    constants::ZERO,
    hash::{HashMethod, hash_u64, sha256}
};

abi MyContract {
    fn sha256_u8(value: u8) -> b256;
    fn sha256_u16(value: u16) -> b256;
    fn sha256_u32(value: u32) -> b256;
    fn sha256_u64(value: u64) -> b256;
    fn sha256_str(value: str[4]) -> b256;
    fn sha256_bool(value: bool) -> b256;
    fn sha256_b256(value: b256) -> b256;
    // Bug when using struct / enum in tuple so using simpler example
    // fn sha256_tuple(value: (Person, Location, u64)) -> b256;
    fn sha256_tuple(value: (bool, u64)) -> b256;
    fn sha256_array(value: [u64;
    2]) -> b256;
    fn sha256_struct(name: str[4], birth_place: bool) -> b256;
    fn sha256_enum(location: bool) -> b256;
    fn get_s256_hash_u64(value: u64) -> b256;
    fn get_k256_hash_u64(value: u64) -> b256;
}

enum Location {
    Earth: (),
    Mars: (),
}

struct Person {
    name: str[4],
    age: u8,
    birth_place: Location,
    stats: Stats,
    alive: bool,
    random_b256: b256,
}

struct Stats {
    strength: u64,
    agility: u64,
}

impl MyContract for Contract {
    fn sha256_u8(value: u8) -> b256 {
        sha256(value)
    }

    fn sha256_u16(value: u16) -> b256 {
        sha256(value)
    }

    fn sha256_u32(value: u32) -> b256 {
        sha256(value)
    }

    fn sha256_u64(value: u64) -> b256 {
        sha256(value)
    }

    fn sha256_str(value: str[4]) -> b256 {
        sha256(value)
    }

    fn sha256_bool(value: bool) -> b256 {
        sha256(value)
    }

    fn sha256_b256(value: b256) -> b256 {
        sha256(value)
    }

    fn sha256_tuple(value: (bool, u64)) -> b256 {
        sha256(value)
    }

    fn sha256_array(value: [u64;
    2]) -> b256 {
        sha256(value)
    }

    fn sha256_struct(name: str[4], birth_place: bool) -> b256 {
        sha256(Person {
            name,
            age: 9000,
            birth_place: if birth_place { Location::Earth } else { Location::Mars },
            stats: Stats {
                strength: 10,
                agility: 9
            },
            alive: true,
            random_b256: ZERO
        })
    }

    fn sha256_enum(location: bool) -> b256 {
        sha256(if location { Location::Earth } else { Location::Mars })
    }

    fn get_s256_hash_u64(value: u64) -> b256 {
        hash_u64(value, HashMethod::Sha256)
    }

    fn get_k256_hash_u64(value: u64) -> b256 {
        hash_u64(value, HashMethod::Keccak256)
    }
}
