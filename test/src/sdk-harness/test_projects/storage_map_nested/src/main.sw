contract;

use std::hash::*;

struct M {
    u: b256,
    v: u64,
}

impl PartialEq for M {
    fn eq(self, other: Self) -> bool {
        self.u == other.u && self.v == other.v
    }
}
impl Eq for M {}

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

impl PartialEq for E {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (E::A(l), E::A(r)) => l == r,
            (E::B(l), E::B(r)) => l == r,
            _ => false,
        }
    }
}
impl Eq for E {}

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

        // These combinations of keys are not set
        assert(storage.nested_map_1.get(2).get(1).get(1).try_read().is_none());
        assert(storage.nested_map_1.get(1).get(2).get(1).try_read().is_none());
        assert(storage.nested_map_1.get(1).get(1).get(2).try_read().is_none());

        let result_1: bool = storage.nested_map_1.get(0).get(0).remove(0);
        assert(result_1);
        assert(storage.nested_map_1.get(0).get(0).get(0).try_read().is_none());

        let result_2: bool = storage.nested_map_1.get(0).get(0).remove(1);
        assert(result_2);
        assert(storage.nested_map_1.get(0).get(0).get(1).try_read().is_none());

        let result_3: bool = storage.nested_map_1.get(0).get(1).remove(0);
        assert(result_3);
        assert(storage.nested_map_1.get(0).get(1).get(0).try_read().is_none());

        let result_4: bool = storage.nested_map_1.get(1).get(1).remove(0);
        assert(result_4);
        assert(storage.nested_map_1.get(1).get(1).get(0).try_read().is_none());
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

        let _0000 = __to_str_array("0000");
        let _0001 = __to_str_array("0001");
        let _0002 = __to_str_array("0002");

        // Map insert via `insert`
        storage.nested_map_2.get((0, 0)).get(_0000).insert(0, m1);
        storage.nested_map_2.get((0, 0)).get(_0001).insert(1, m2);
        storage.nested_map_2.get((0, 1)).get(_0000).insert(0, m1);
        storage.nested_map_2.get((0, 1)).get(_0001).insert(1, m2);

        // Map insert via `get`
        assert(storage.nested_map_2.get((0, 0)).get(_0000).get(0).read() == m1);
        assert(storage.nested_map_2.get((0, 0)).get(_0001).get(1).read() == m2);
        assert(storage.nested_map_2.get((0, 1)).get(_0000).get(0).read() == m1);
        assert(storage.nested_map_2.get((0, 1)).get(_0001).get(1).read() == m2);

        // These combinations of keys are not set
        assert(storage.nested_map_2.get((2, 0)).get(_0001).get(1).try_read().is_none());
        assert(storage.nested_map_2.get((1, 1)).get(_0002).get(0).try_read().is_none());
        assert(storage.nested_map_2.get((1, 1)).get(_0001).get(2).try_read().is_none());

        let result_1: bool = storage.nested_map_2.get((0, 0)).get(_0000).remove(0);
        assert(result_1);
        assert(storage.nested_map_2.get((0, 0)).get(_0000).get(0).try_read().is_none());

        let result_2: bool = storage.nested_map_2.get((0, 0)).get(_0001).remove(1);
        assert(result_2);
        assert(storage.nested_map_2.get((0, 0)).get(_0001).get(1).try_read().is_none());

        let result_3: bool = storage.nested_map_2.get((0, 1)).get(_0000).remove(0);
        assert(result_3);
        assert(storage.nested_map_2.get((0, 1)).get(_0000).get(0).try_read().is_none());

        let result_4: bool = storage.nested_map_2.get((0, 1)).get(_0001).remove(1);
        assert(result_4);
        assert(storage.nested_map_2.get((0, 1)).get(_0001).get(1).try_read().is_none());
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

        // These combinations of keys are not set
        assert(storage.nested_map_3.get(2).get(m2).get(1).try_read().is_none());
        assert(
            storage
                .nested_map_3
                .get(1)
                .get(M {
                    u: b256::zero(),
                    v: 3,
                })
                .get(1)
                .try_read()
                .is_none(),
        );
        assert(storage.nested_map_3.get(1).get(m2).get(2).try_read().is_none());

        let result_1: bool = storage.nested_map_3.get(0).get(m1).remove(0);
        assert(result_1);
        assert(storage.nested_map_3.get(0).get(m1).get(0).try_read().is_none());

        let result_2: bool = storage.nested_map_3.get(0).get(m2).remove(1);
        assert(result_2);
        assert(storage.nested_map_3.get(0).get(m2).get(1).try_read().is_none());

        let result_3: bool = storage.nested_map_3.get(1).get(m1).remove(0);
        assert(result_3);
        assert(storage.nested_map_3.get(1).get(m1).get(0).try_read().is_none());

        let result_4: bool = storage.nested_map_3.get(1).get(m2).remove(1);
        assert(result_4);
        assert(storage.nested_map_3.get(1).get(m2).get(1).try_read().is_none());
    }
}
