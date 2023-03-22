contract;

use core::experimental::storage::*;
use std::experimental::storage::*;

pub struct S {
    x: u64,
    y: u64,
    z: b256,
    t: T,
}

pub struct T {
    x: u64,
    y: u64,
    z: b256,
    boolean: bool,
    int8: u8,
    int16: u16,
    int32: u32,
}

pub enum E {
    A: u64,
    B: T,
}

impl core::ops::Eq for T {
    fn eq(self, other: Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z && self.boolean == other.boolean && self.int8 == other.int8 && self.int16 == other.int16 && self.int32 == other.int32
    }
}

impl core::ops::Eq for S {
    fn eq(self, other: Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z && self.t == other.t
    }
}

impl core::ops::Eq for E {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (E::A(l), E::A(r)) => l == r,
            (E::B(l), E::B(r)) => l == r,
            _ => false,
        }
    }
}

storage {
    x: u64 = 64,
    y: b256 = 0x0101010101010101010101010101010101010101010101010101010101010101,
    s: S = S {
        x: 1,
        y: 2,
        z: 0x0000000000000000000000000000000000000000000000000000000000000003,
        t: T {
            x: 4,
            y: 5,
            z: 0x0000000000000000000000000000000000000000000000000000000000000006,
            boolean: true,
            int8: 7,
            int16: 8,
            int32: 9,
        },
    },
    boolean: bool = true,
    int8: u8 = 8,
    int16: u16 = 16,
    int32: u32 = 32,
    e: E = E::B(T {
        x: 1,
        y: 2,
        z: 0x0000000000000000000000000000000000000000000000000000000000000003,
        boolean: true,
        int8: 4,
        int16: 5,
        int32: 6,
    }),
    e2: E = E::A(777),
    string: str[40] = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
}

abi ExperimentalStorageInitTest {
    #[storage(read)]
    fn test_initializers() -> bool;
}

impl ExperimentalStorageInitTest for Contract {
    #[storage(read)]
    fn test_initializers() -> bool { /* Initializer values */
        let x: u64 = 64;
        let y: b256 = 0x0101010101010101010101010101010101010101010101010101010101010101;
        let s: S = S {
            x: 1,
            y: 2,
            z: 0x0000000000000000000000000000000000000000000000000000000000000003,
            t: T {
                x: 4,
                y: 5,
                z: 0x0000000000000000000000000000000000000000000000000000000000000006,
                boolean: true,
                int8: 7,
                int16: 8,
                int32: 9,
            },
        };
        let boolean: bool = true;
        let int8: u8 = 8;
        let int16: u16 = 16;
        let int32: u32 = 32;
        let e: E = E::B(T {
            x: 1,
            y: 2,
            z: 0x0000000000000000000000000000000000000000000000000000000000000003,
            boolean: true,
            int8: 4,
            int16: 5,
            int32: 6,
        });
        let e2: E = E::A(777);
        let string: str[40] = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

        assert(storage.x.read().unwrap() == x);
        assert(storage.y.read().unwrap() == y);
        assert(storage.s.read().unwrap() == s);
        assert(storage.boolean.read().unwrap() == boolean);
        assert(storage.int8.read().unwrap() == int8);
        assert(storage.int16.read().unwrap() == int16);
        assert(storage.int32.read().unwrap() == int32);
        assert(storage.s.x.read().unwrap() == s.x);
        assert(storage.s.y.read().unwrap() == s.y);
        assert(storage.s.z.read().unwrap() == s.z);
        assert(storage.s.t.read().unwrap() == s.t);
        assert(storage.s.t.x.read().unwrap() == s.t.x);
        assert(storage.s.t.y.read().unwrap() == s.t.y);
        assert(storage.s.t.z.read().unwrap() == s.t.z);
        assert(storage.s.t.boolean.read().unwrap() == s.t.boolean);
        assert(storage.s.t.int8.read().unwrap() == s.t.int8);
        assert(storage.s.t.int16.read().unwrap() == s.t.int16);
        assert(storage.s.t.int32.read().unwrap() == s.t.int32);
        assert(storage.e.read().unwrap() == e);
        assert(storage.e2.read().unwrap() == e2);
        assert(std::hash::sha256(storage.string.read().unwrap()) == std::hash::sha256(string));
        true
    }
}
