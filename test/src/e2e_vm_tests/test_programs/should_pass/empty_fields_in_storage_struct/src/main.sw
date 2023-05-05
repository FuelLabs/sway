contract;

use std::storage::storage_map::*;

abi ReproAttempt {
    #[storage(read, write)]
    fn foo_insert(key: u64, value: u64);

    #[storage(read)]
    fn foo_get(key: u64) -> Option<u64>;

    #[storage(read)]
    fn foo_len() -> u64;

    #[storage(read, write)]
    fn bar_insert(key: u64, value: u64);

    #[storage(read)]
    fn bar_get(key: u64) -> Option<u64>;

    #[storage(read)]
    fn bar_len() -> u64;
}

struct StructOfStorageMaps {
    foo: StorageMap<u64, u64>,
    bar: StorageMap<u64, u64>,
}

storage {
    struct_of_maps: StructOfStorageMaps = StructOfStorageMaps {
        foo: StorageMap {},
        bar: StorageMap {},
    },
}

impl ReproAttempt for Contract {
    #[storage(read, write)]
    fn foo_insert(key: u64, value: u64) {
        storage.struct_of_maps.foo.insert(key, value);
    }

    #[storage(read)]
    fn foo_get(key: u64) -> Option<u64> {
        storage.struct_of_maps.foo.get(key).try_read()
    }

    #[storage(read, write)]
    fn bar_insert(key: u64, value: u64) {
        storage.struct_of_maps.bar.insert(key, value);
    }

    #[storage(read)]
    fn bar_get(key: u64) -> Option<u64> {
        storage.struct_of_maps.bar.get(key).try_read()
    }

    #[storage(read)]    
    fn foo_len() -> u64 {
        storage.struct_of_maps.foo.len().try_read()
    }

    #[storage(read)]
    fn bar_len() -> u64 {
        storage.struct_of_maps.bar.len().try_read()
    }
}

#[test()]
fn test_read_write() {
    let repro = abi(ReproAttempt, CONTRACT_ID);

    assert(repro.foo_get(1).is_none());
    assert(repro.foo_len() == 0);
    assert(repro.bar_get(1).is_none());
    assert(repro.bar_len() == 0);

    repro.foo_insert(1, 2);

    assert(repro.foo_get(1).unwrap() == 2);
    assert(repro.foo_len() == 1);

    assert(repro.bar_get(1).is_none());
    assert(repro.bar_len() == 0);
}
