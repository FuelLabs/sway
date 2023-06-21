contract;

use std::constants::ZERO_B256;
use std::hash::*;

struct M {
    u: b256,
    v: u64,
}

impl core::ops::Eq for M {
    fn eq(self, other: Self) -> bool {
        self.u == other.u && self.v == other.v
    }
}

impl Hash for M {
    fn hash(self, ref mut state: Hasher) {
        self.u.hash(state);
        self.v.hash(state);
    }
}

pub enum E {
    A: u64,
    B: b256,
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

impl Hash for str[4] {
    fn hash(self, ref mut state: Hasher) {
        state.write_str(self);
    }
}

impl Hash for (u64, u64) {
    fn hash(self, ref mut state: Hasher) {
        self.0.hash(state);
        self.1.hash(state);
    }
}

storage {
    nested_map_1: StorageMap<u64, StorageMap<u64, StorageMap<u64, u64>>> = StorageMap::<u64, StorageMap<u64, StorageMap<u64, u64>>> {},
    nested_map_2: StorageMap<(u64, u64), StorageMap<str[4], StorageMap<u64, M>>> = StorageMap::<(u64, u64), StorageMap<str[4], StorageMap<u64, M>>> {},
    nested_map_3: StorageMap<u64, StorageMap<M, StorageMap<u64, E>>> = StorageMap::<u64, StorageMap<M, StorageMap<u64, E>>> {},
}

abi ExperimentalStorageTest {
    #[storage(read, write)]
    fn nested_map_1_access();

    #[storage(read, write)]
    fn nested_map_2_access();

    #[storage(read, write)]
    fn nested_map_3_access();
}

impl ExperimentalStorageTest for Contract {
    #[storage(read, write)]
    fn nested_map_1_access() {
        // Map insert via `insert`
        storage.nested_map_1.get(0).get(0).insert(0, 1);
        storage.nested_map_1.get(0).get(0).insert(1, 2);
        storage.nested_map_1.get(0).get(1).insert(0, 3);
        storage.nested_map_1.get(0).get(1).insert(1, 4);
        storage.nested_map_1.get(1).get(0).insert(0, 5);
        storage.nested_map_1.get(1).get(0).insert(1, 6);
        storage.nested_map_1.get(1).get(1).insert(0, 7);
        storage.nested_map_1.get(1).get(1).insert(1, 8);

        // Map access via `get`
        assert(storage.nested_map_1.get(0).get(0).get(0).read() == 1);
        assert(storage.nested_map_1.get(0).get(0).get(1).read() == 2);
        assert(storage.nested_map_1.get(0).get(1).get(0).read() == 3);
        assert(storage.nested_map_1.get(0).get(1).get(1).read() == 4);
        assert(storage.nested_map_1.get(1).get(0).get(0).read() == 5);
        assert(storage.nested_map_1.get(1).get(0).get(1).read() == 6);
        assert(storage.nested_map_1.get(1).get(1).get(0).read() == 7);
        assert(storage.nested_map_1.get(1).get(1).get(1).read() == 8);

        // Thes combinations of keys are not set
        assert(storage.nested_map_1.get(2).get(1).get(1).try_read().is_none());
        assert(storage.nested_map_1.get(1).get(2).get(1).try_read().is_none());
        assert(storage.nested_map_1.get(1).get(1).get(2).try_read().is_none());
    }

    #[storage(read, write)]
    fn nested_map_2_access() {
        let m1 = M {
            u: 0x1111111111111111111111111111111111111111111111111111111111111111,
            v: 1,
        };
        let m2 = M {
            u: 0x2222222222222222222222222222222222222222222222222222222222222222,
            v: 2,
        };

        // Map insert via `insert`
        storage.nested_map_2.get((0, 0)).get("0000").insert(0, m1);
        storage.nested_map_2.get((0, 0)).get("0001").insert(1, m2);
        storage.nested_map_2.get((0, 1)).get("0000").insert(0, m1);
        storage.nested_map_2.get((0, 1)).get("0001").insert(1, m2);

        // Map insert via `get`
        assert(storage.nested_map_2.get((0, 0)).get("0000").get(0).read() == m1);
        assert(storage.nested_map_2.get((0, 0)).get("0001").get(1).read() == m2);
        assert(storage.nested_map_2.get((0, 1)).get("0000").get(0).read() == m1);
        assert(storage.nested_map_2.get((0, 1)).get("0001").get(1).read() == m2);

        // Thes combinations of keys are not set
        assert(storage.nested_map_2.get((2, 0)).get("0001").get(1).try_read().is_none());
        assert(storage.nested_map_2.get((1, 1)).get("0002").get(0).try_read().is_none());
        assert(storage.nested_map_2.get((1, 1)).get("0001").get(2).try_read().is_none());
    }

    #[storage(read, write)]
    fn nested_map_3_access() {
        let m1 = M {
            u: 0x1111111111111111111111111111111111111111111111111111111111111111,
            v: 1,
        };
        let m2 = M {
            u: 0x2222222222222222222222222222222222222222222222222222222222222222,
            v: 2,
        };
        let e1 = E::A(42);
        let e2 = E::B(0x3333333333333333333333333333333333333333333333333333333333333333);

        // Map insert via `insert`
        storage.nested_map_3.get(0).get(m1).insert(0, e1);
        storage.nested_map_3.get(0).get(m2).insert(1, e2);
        storage.nested_map_3.get(0).get(m1).insert(0, e1);
        storage.nested_map_3.get(0).get(m2).insert(1, e2);
        storage.nested_map_3.get(1).get(m1).insert(0, e1);
        storage.nested_map_3.get(1).get(m2).insert(1, e2);
        storage.nested_map_3.get(1).get(m1).insert(0, e1);
        storage.nested_map_3.get(1).get(m2).insert(1, e2);

        // Map insert via `get`
        assert(storage.nested_map_3.get(0).get(m1).get(0).read() == e1);
        assert(storage.nested_map_3.get(0).get(m2).get(1).read() == e2);
        assert(storage.nested_map_3.get(0).get(m1).get(0).read() == e1);
        assert(storage.nested_map_3.get(0).get(m2).get(1).read() == e2);
        assert(storage.nested_map_3.get(1).get(m1).get(0).read() == e1);
        assert(storage.nested_map_3.get(1).get(m2).get(1).read() == e2);
        assert(storage.nested_map_3.get(1).get(m1).get(0).read() == e1);
        assert(storage.nested_map_3.get(1).get(m2).get(1).read() == e2);

        // Thes combinations of keys are not set
        assert(storage.nested_map_3.get(2).get(m2).get(1).try_read().is_none());
        assert(storage.nested_map_3.get(1).get(M {
            u: ZERO_B256,
            v: 3,
        }).get(1).try_read().is_none());
        assert(storage.nested_map_3.get(1).get(m2).get(2).try_read().is_none());
    }
}
