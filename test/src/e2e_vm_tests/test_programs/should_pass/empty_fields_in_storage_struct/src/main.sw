contract;

use std::bytes::Bytes;
use std::storage::storage_bytes::*;
use std::storage::storage_map::*;

abi ReproAttempt {
    #[storage(read, write)]
    fn bytes_foo_store(bytes: Bytes);

    #[storage(read)]
    fn bytes_foo_get() -> Option<Bytes>;

    #[storage(read)]
    fn bytes_foo_len() -> u64;

    #[storage(read, write)]
    fn bytes_bar_store(bytes: Bytes);

    #[storage(read)]
    fn bytes_bar_get() -> Option<Bytes>;

    #[storage(read)]
    fn bytes_bar_len() -> u64;

    #[storage(read, write)]
    fn map_foo_insert(key: u64, value: u64);

    #[storage(read)]
    fn map_foo_get(key: u64) -> Option<u64>;

    #[storage(read, write)]
    fn map_bar_insert(key: u64, value: u64);

    #[storage(read)]
    fn map_bar_get(key: u64) -> Option<u64>;
}

struct StructOfStorageBytes {
    foo: StorageBytes,
    bar: StorageBytes,
}

struct StructOfStorageMaps {
    foo: StorageMap<u64, u64>,
    bar: StorageMap<u64, u64>,
}

storage {
    struct_of_bytes: StructOfStorageBytes = StructOfStorageBytes {
        foo: StorageBytes {},
        bar: StorageBytes {},
    },
    struct_of_maps: StructOfStorageMaps = StructOfStorageMaps {
        foo: StorageMap {},
        bar: StorageMap {},
    },
}

impl ReproAttempt for Contract {
    #[storage(read, write)]
    fn bytes_foo_store(bytes: Bytes) {
        storage.struct_of_bytes.foo.store(bytes);
    }

    #[storage(read)]
    fn bytes_foo_get() -> Option<Bytes> {
        storage.struct_of_bytes.foo.load()
    }

    #[storage(read)]
    fn bytes_foo_len() -> u64 {
        storage.struct_of_bytes.foo.len()
    }

    #[storage(read, write)]
    fn bytes_bar_store(bytes: Bytes) {
        storage.struct_of_bytes.bar.store(bytes);
    }

    #[storage(read)]
    fn bytes_bar_get() -> Option<Bytes> {
        storage.struct_of_bytes.bar.load()
    }

    #[storage(read)]
    fn bytes_bar_len() -> u64 {
        storage.struct_of_bytes.bar.len()
    }

    #[storage(read, write)]
    fn map_foo_insert(key: u64, value: u64) {
        storage.struct_of_maps.foo.insert(key, value);
    }

    #[storage(read)]
    fn map_foo_get(key: u64) -> Option<u64> {
        storage.struct_of_maps.foo.get(key).try_read()
    }

    #[storage(read, write)]
    fn map_bar_insert(key: u64, value: u64) {
        storage.struct_of_maps.bar.insert(key, value);
    }

    #[storage(read)]
    fn map_bar_get(key: u64) -> Option<u64> {
        storage.struct_of_maps.bar.get(key).try_read()
    }
}

#[test()]
fn test_read_write_bytes() {
    let repro = abi(ReproAttempt, CONTRACT_ID);

    let mut my_bytes = Bytes::new();
    my_bytes.push(1_u8);
    my_bytes.push(2_u8);

    assert(repro.bytes_foo_get().is_none());
    assert(repro.bytes_bar_get().is_none());
    assert(repro.bytes_foo_len() == 0);
    assert(repro.bytes_bar_len() == 0);

    repro.bytes_foo_store(my_bytes);

    assert(repro.bytes_foo_get().unwrap() == my_bytes);
    assert(repro.bytes_bar_get().is_none());
    assert(repro.bytes_foo_len() == 2);
    assert(repro.bytes_bar_len() == 0);
}

#[test()]
fn test_read_write_map() {
    let repro = abi(ReproAttempt, CONTRACT_ID);

    assert(repro.map_foo_get(1).is_none());
    assert(repro.map_bar_get(1).is_none());

    repro.map_foo_insert(1, 2);

    assert(repro.map_foo_get(1).unwrap() == 2);
    assert(repro.map_bar_get(1).is_none());
}
