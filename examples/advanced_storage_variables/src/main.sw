contract;

use std::{bytes::Bytes, string::String,};

// ANCHOR: temp_hash_import
use std::hash::Hash;
// ANCHOR: temp_hash_import

// ANCHOR: storage_vec_import
use std::storage::storage_vec::*;
// ANCHOR: storage_vec_import

// ANCHOR: storage_bytes_import
use std::storage::storage_bytes::*;
// ANCHOR: storage_bytes_import

// ANCHOR: storage_bytes_import
use std::storage::storage_string::*;
// ANCHOR: storage_bytes_import

// ANCHOR: advanced_storage_declaration
storage {
    storage_map: StorageMap<u64, bool> = StorageMap {},
    storage_vec: StorageVec<b256> = StorageVec {},
    storage_string: StorageString = StorageString {},
    storage_bytes: StorageBytes = StorageBytes {},
}
// ANCHOR_END: advanced_storage_declaration

abi StorageExample {
    #[storage(write)]
    fn store_map();
    #[storage(read)]
    fn get_map();
    #[storage(write)]
    fn store_vec();
    #[storage(read, write)]
    fn get_vec();
    #[storage(write)]
    fn store_string();
    #[storage(read)]
    fn get_string();
    #[storage(write)]
    fn store_bytes();
    #[storage(read)]
    fn get_bytes();
}

impl StorageExample for Contract {
    #[storage(write)]
    fn store_map() {
        // ANCHOR: map_storage_write
        storage.storage_map.insert(12, true);
        storage.storage_map.insert(59, false);

        // try_insert() will only insert if a value does not already exist for a key.
        let result = storage.storage_map.try_insert(103, true);
        assert(result.is_ok());
        // ANCHOR_END: map_storage_write
    }
    #[storage(read)]
    fn get_map() {
        // ANCHOR: map_storage_read
        // Access directly
        let stored_val1: bool = storage.storage_map.get(12).try_read().unwrap_or(false);

        // First get the storage key and then access the value.
        let storage_key2: StorageKey<bool> = storage.storage_map.get(59);
        let stored_val2: bool = storage_key2.try_read().unwrap_or(false);

        // Unsafely access the value.
        let stored_val3: bool = storage.storage_map.get(103).read();
        // ANCHOR_END: map_storage_read
    }

    #[storage(write)]
    fn store_vec() {
        // ANCHOR: vec_storage_write
        storage
            .storage_vec
            .push(0x1111111111111111111111111111111111111111111111111111111111111111);
        storage
            .storage_vec
            .push(0x0000000000000000000000000000000000000000000000000000000000000001);
        storage
            .storage_vec
            .push(0x0000000000000000000000000000000000000000000000000000000000000002);

        // Set will overwrite the element stored at the given index.
        storage.storage_vec.set(2, b256::zero());
        // ANCHOR_END: vec_storage_write
    }
    #[storage(read, write)]
    fn get_vec() {
        // ANCHOR: vec_storage_read
        // Method 1: Access the element directly
        // Note: get() does not remove the element from the vec.
        let stored_val1: b256 = storage.storage_vec.get(0).unwrap().try_read().unwrap_or(b256::zero());

        // Method 2: First get the storage key and then access the value.
        let storage_key2: StorageKey<b256> = storage.storage_vec.get(1).unwrap();
        let stored_val2: b256 = storage_key2.try_read().unwrap_or(b256::zero());

        // pop() will remove the last element from the vec.
        let length: u64 = storage.storage_vec.len();
        let stored_val3: b256 = storage.storage_vec.pop().unwrap();
        assert(length != storage.storage_vec.len());
        // ANCHOR_END: vec_storage_read
    }

    #[storage(write)]
    fn store_string() {
        // ANCHOR: string_storage_write
        let my_string = String::from_ascii_str("Fuel is blazingly fast");
        storage.storage_string.write_slice(my_string);
        // ANCHOR_END: string_storage_write
    }
    #[storage(read)]
    fn get_string() {
        // ANCHOR: string_storage_read
        let stored_string: String = storage.storage_string.read_slice().unwrap();
        // ANCHOR_END: string_storage_read
    }

    #[storage(write)]
    fn store_bytes() {
        // ANCHOR: bytes_storage_write
        // Setup Bytes
        let mut my_bytes = Bytes::new();
        my_bytes.push(1u8);
        my_bytes.push(2u8);
        my_bytes.push(3u8);

        // Write to storage
        storage.storage_bytes.write_slice(my_bytes);
        // ANCHOR_END: bytes_storage_write
    }
    #[storage(read)]
    fn get_bytes() {
        // ANCHOR: bytes_storage_read
        let stored_bytes: Bytes = storage.storage_bytes.read_slice().unwrap();
        // ANCHOR_END: bytes_storage_read
    }
}
