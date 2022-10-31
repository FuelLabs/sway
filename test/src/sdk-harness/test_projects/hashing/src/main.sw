contract;

use std::hash::{keccak256, sha256};
use core::num::*;

abi MyContract {
    fn sha256_u8(value: u8) -> b256;
    fn sha256_u16(value: u16) -> b256;
    fn sha256_u32(value: u32) -> b256;
    fn sha256_u64(value: u64) -> b256;
    fn sha256_bool(value: bool) -> b256;
    fn sha256_str(value: str[4]) -> b256;
    fn sha256_b256(value: b256) -> b256;
    fn sha256_tuple(value: (bool, u64)) -> b256;
    fn sha256_array(value1: u64, value2: u64) -> b256;
    fn sha256_enum(location: bool) -> b256;
    fn sha256_struct(location: bool) -> b256;

    fn keccak256_u8(value: u8) -> b256;
    fn keccak256_u16(value: u16) -> b256;
    fn keccak256_u32(value: u32) -> b256;
    fn keccak256_u64(value: u64) -> b256;
    fn keccak256_bool(value: bool) -> b256;
    fn keccak256_str(value: str[4]) -> b256;
    fn keccak256_b256(value: b256) -> b256;
    fn keccak256_tuple(value: (bool, u64)) -> b256;
    fn keccak256_array(value1: u64, value2: u64) -> b256;
    fn keccak256_enum(location: bool) -> b256;
    fn keccak256_struct(location: bool) -> b256;
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

    fn sha256_bool(value: bool) -> b256 {
        sha256(value)
    }

    fn sha256_str(value: str[4]) -> b256 {
        sha256(value)
    }

    fn sha256_b256(value: b256) -> b256 {
        sha256(value)
    }

    fn sha256_tuple(value: (bool, u64)) -> b256 {
        sha256(value)
    }

    fn sha256_array(value1: u64, value2: u64) -> b256 {
        sha256([value1, value2])
    }

    fn sha256_enum(location: bool) -> b256 {
        sha256(if location {
            Location::Earth
        } else {
            Location::Mars
        })
    }

    fn sha256_struct(location: bool) -> b256 {
        sha256(Person {
            name: "John",
            age: 18,
            birth_place: if location {
                Location::Earth
            } else {
                Location::Mars
            },
            stats: Stats {
                strength: 10,
                agility: 9,
            },
            alive: true,
            random_b256: b256::min(),
        })
    }

    fn keccak256_u8(value: u8) -> b256 {
        keccak256(value)
    }

    fn keccak256_u16(value: u16) -> b256 {
        keccak256(value)
    }

    fn keccak256_u32(value: u32) -> b256 {
        keccak256(value)
    }

    fn keccak256_u64(value: u64) -> b256 {
        keccak256(value)
    }

    fn keccak256_bool(value: bool) -> b256 {
        keccak256(value)
    }

    fn keccak256_str(value: str[4]) -> b256 {
        keccak256(value)
    }

    fn keccak256_b256(value: b256) -> b256 {
        keccak256(value)
    }

    fn keccak256_tuple(value: (bool, u64)) -> b256 {
        keccak256(value)
    }

    fn keccak256_array(value1: u64, value2: u64) -> b256 {
        keccak256([value1, value2])
    }

    fn keccak256_enum(location: bool) -> b256 {
        keccak256(if location {
            Location::Earth
        } else {
            Location::Mars
        })
    }

    fn keccak256_struct(location: bool) -> b256 {
        keccak256(Person {
            name: "John",
            age: 18,
            birth_place: if location {
                Location::Earth
            } else {
                Location::Mars
            },
            stats: Stats {
                strength: 10,
                agility: 9,
            },
            alive: true,
            random_b256: b256::min(),
        })
    }
}
