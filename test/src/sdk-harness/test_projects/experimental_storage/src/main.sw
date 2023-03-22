contract;

use core::experimental::storage::*;
use std::experimental::storage::*;

struct M {
    u: b256,
    v: u64,
}

struct T {
    x: u64,
    y: b256,
    z: M,
}

struct S {
    a: u64,
    b: b256,
    c: T,
    d: b256,
}

struct Simple {
    x: u64,
    y: u64,
    b: b256,
    z: u64,
    w: u64,
}

impl core::ops::Eq for M {
    fn eq(self, other: Self) -> bool {
        self.u == other.u && self.v == other.v
    }
}

impl core::ops::Eq for T {
    fn eq(self, other: Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

impl core::ops::Eq for S {
    fn eq(self, other: Self) -> bool {
        self.a == other.a && self.b == other.b && self.c == other.c && self.d == other.d
    }
}

struct S2 {
    map0: StorageMap<u64, u64>,
    map1: StorageMap<u64, u64>,
}

storage {
    x: u64 = 0,
    y: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000,
    map: StorageMap<u64, u64> = StorageMap {},
    s: S = S {
        a: 0,
        b: 0x0000000000000000000000000000000000000000000000000000000000000000,
        c: T {
            x: 0,
            y: 0x0000000000000000000000000000000000000000000000000000000000000000,
            z: M {
                u: 0x0000000000000000000000000000000000000000000000000000000000000000,
                v: 0,
            },
        },
        d: 0x0000000000000000000000000000000000000000000000000000000000000000,
    },
    s2: S2 = S2 {
        map0: StorageMap {},
        map1: StorageMap {},
    },
    simple: Simple = Simple {
        x: 0,
        y: 0,
        b: 0x0000000000000000000000000000000000000000000000000000000000000000,
        z: 0,
        w: 0,
    },
    map_in_map: StorageMap<u64, StorageHandle<StorageMap<u64, u64>>> = StorageMap {},
}

abi ExperimentalStorageTest {
    #[storage(read, write)]
    fn write_and_read_u64(input: u64) -> u64;

    #[storage(read, write)]
    fn write_and_read_b256(input: b256) -> b256;

    #[storage(read, write)]
    fn write_and_read_struct_simple(s: Simple) -> Simple;

    #[storage(read, write)]
    fn write_and_read_struct_1(s: S) -> S;

    #[storage(read, write)]
    fn write_and_read_struct_2(s: S) -> S;

    #[storage(read)]
    fn map_read(key: u64) -> Option<u64>;

    #[storage(read, write)]
    fn map_write(key: u64, value: u64);

    #[storage(read)]
    fn map_in_struct_read(key: (u64, u64)) -> (Option<u64>, Option<u64>);

    #[storage(read, write)]
    fn map_in_struct_write(key: (u64, u64), value: (u64, u64));

    #[storage(read, write)]
    fn map_in_map_access();
}

impl ExperimentalStorageTest for Contract {
    #[storage(read, write)]
    fn write_and_read_u64(input: u64) -> u64 {
        let r = storage.x;
        r.write(input);
        r.read().unwrap()
    }

    #[storage(read, write)]
    fn write_and_read_b256(input: b256) -> b256 {
        storage.y.write(input);
        storage.y.read().unwrap()
    }

    #[storage(read, write)]
    fn write_and_read_struct_simple(simple: Simple) -> Simple {
        // Make sure that writing `b` does not erase `z`. `z` comes right after `b` in the storage 
        // slot where the second half of `simple` is stored
        storage.simple.z.write(simple.z);
        storage.simple.b.write(simple.b);
        storage.simple.read().unwrap()
    }

    #[storage(read, write)]
    fn write_and_read_struct_1(s: S) -> S {
        // Granular read and write
        storage.s.a.write(s.a);
        storage.s.b.write(s.b);
        storage.s.c.x.write(s.c.x);
        storage.s.c.y.write(s.c.y);
        storage.s.c.z.write(s.c.z);
        storage.s.d.write(s.d);

        assert(S {
            a: storage.s.a.read().unwrap(),
            b: storage.s.b.read().unwrap(),
            c: T {
                x: storage.s.c.x.read().unwrap(),
                y: storage.s.c.y.read().unwrap(),
                z: M {
                    u: storage.s.c.z.u.read().unwrap(),
                    v: storage.s.c.z.v.read().unwrap(),
                },
            },
            d: storage.s.d.read().unwrap(),
        } == s);

        // Semi-granular write, granular read
        storage.s.a.write(s.a);
        storage.s.b.write(s.b);
        storage.s.c.write(s.c);
        storage.s.d.write(s.d);

        assert(S {
            a: storage.s.a.read().unwrap(),
            b: storage.s.b.read().unwrap(),
            c: T {
                x: storage.s.c.x.read().unwrap(),
                y: storage.s.c.y.read().unwrap(),
                z: M {
                    u: storage.s.c.z.u.read().unwrap(),
                    v: storage.s.c.z.v.read().unwrap(),
                },
            },
            d: storage.s.d.read().unwrap(),
        } == s);

        storage.s.read().unwrap()
    }

    #[storage(read, write)]
    fn write_and_read_struct_2(s: S) -> S {
        // Granular write, semi-granular read
        storage.s.a.write(s.a);
        storage.s.b.write(s.b);
        storage.s.c.x.write(s.c.x);
        storage.s.c.y.write(s.c.y);
        storage.s.c.z.write(s.c.z);
        storage.s.d.write(s.d);

        assert(S {
            a: storage.s.a.read().unwrap(),
            b: storage.s.b.read().unwrap(),
            c: storage.s.c.read().unwrap(),
            d: storage.s.d.read().unwrap(),
        } == s);

        // Coarse write and read
        storage.s.write(s);

        storage.s.read().unwrap()
    }

    #[storage(read)]
    fn map_read(key: u64) -> Option<u64> {
        storage.map.get(key)
    }

    #[storage(read, write)]
    fn map_write(key: u64, value: u64) {
        storage.map.insert(key, value);
    }

    #[storage(read)]
    fn map_in_struct_read(key: (u64, u64)) -> (Option<u64>, Option<u64>) {
        (storage.s2.map0.get(key.0), storage.s2.map1.get(key.1))
    }

    #[storage(read, write)]
    fn map_in_struct_write(key: (u64, u64), value: (u64, u64)) {
        storage.s2.map0.insert(key.0, value.0);
        storage.s2.map1.insert(key.1, value.1);
    }

    #[storage(read, write)]
    fn map_in_map_access() {
        storage.map_in_map.insert(0, StorageHandle {
            key: std::hash::sha256((storage.map_in_map, 0)),
            offset: 0,
        });

        storage.map_in_map.insert(1, StorageHandle {
            key: std::hash::sha256((storage.map_in_map, 1)),
            offset: 0,
        });

        storage.map_in_map.get(0).unwrap().insert(1, 42);
        assert(storage.map_in_map.get(0).unwrap().get(1).unwrap() == 42);

        storage.map_in_map.get(1).unwrap().insert(1, 24);
        assert(storage.map_in_map.get(1).unwrap().get(1).unwrap() == 24);
    }
}
