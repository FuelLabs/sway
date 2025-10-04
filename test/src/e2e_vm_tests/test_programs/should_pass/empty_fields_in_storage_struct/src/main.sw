contract;

use std::bytes::Bytes;
use std::storage::storage_bytes::*;
use std::storage::storage_map::*;
use std::storage::storage_vec::*;
use std::hash::*;


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

    #[storage(read, write)]
    fn vec_foo_push(value: u64);

    #[storage(read)]
    fn vec_foo_get(index: u64) -> Option<u64>;

    #[storage(read, write)]
    fn vec_bar_push(value: u64);

    #[storage(read)]
    fn vec_bar_get(index: u64) -> Option<u64>;

    #[storage(read)]
    fn vec_foo_len() -> u64;

    #[storage(read)]
    fn vec_bar_len() -> u64;
}

struct StructOfStorageBytes {
    foo: StorageBytes,
    bar: StorageBytes,
}

struct StructOfStorageMaps {
    foo: StorageMap<u64, u64>,
    bar: StorageMap<u64, u64>,
}

struct StructOfStorageVecs {
    foo: StorageVec<u64>,
    bar: StorageVec<u64>,
}

storage {
    struct_of_bytes: StructOfStorageBytes = StructOfStorageBytes {
        foo: StorageBytes {},
        bar: StorageBytes {},
    },
    struct_of_maps: StructOfStorageMaps = StructOfStorageMaps {
        foo: StorageMap::<u64, u64> {},
        bar: StorageMap::<u64, u64> {},
    },
    struct_of_vecs: StructOfStorageVecs = StructOfStorageVecs {
        foo: StorageVec {},
        bar: StorageVec {},
    },
}

impl ReproAttempt for Contract {
    #[storage(read, write)]
    fn bytes_foo_store(bytes: Bytes) {
        storage.struct_of_bytes.foo.write_slice(bytes);
    }

    #[storage(read)]
    fn bytes_foo_get() -> Option<Bytes> {
        storage.struct_of_bytes.foo.read_slice()
    }

    #[storage(read)]
    fn bytes_foo_len() -> u64 {
        storage.struct_of_bytes.foo.len()
    }

    #[storage(read, write)]
    fn bytes_bar_store(bytes: Bytes) {
        storage.struct_of_bytes.bar.write_slice(bytes);
    }

    #[storage(read)]
    fn bytes_bar_get() -> Option<Bytes> {
        storage.struct_of_bytes.bar.read_slice()
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

    #[storage(read, write)]
    fn vec_foo_push(value: u64) {
        storage.struct_of_vecs.foo.push(value)
    }

    #[storage(read)]
    fn vec_foo_get(index: u64) -> Option<u64> {
        match storage.struct_of_vecs.foo.get(index) {
            Option::Some(key) => {
                key.try_read()
            },
            Option::None => Option::None,
        }
    }

    #[storage(read, write)]
    fn vec_bar_push(value: u64) {
        storage.struct_of_vecs.bar.push(value)
    }

    #[storage(read)]
    fn vec_bar_get(index: u64) -> Option<u64> {
        match storage.struct_of_vecs.bar.get(index) {
            Option::Some(key) => {
                key.try_read()
            },
            Option::None => Option::None,
        }
    }

    #[storage(read)]
    fn vec_foo_len() -> u64 {
        storage.struct_of_vecs.foo.len()
    }

    #[storage(read)]
    fn vec_bar_len() -> u64 {
        storage.struct_of_vecs.bar.len()
    }
}

#[test()]
fn test_read_write_bytes() {


}

#[test()]
fn test_read_write_map() {
    
}

#[test()]
fn test_read_write_vec() {
    
}
