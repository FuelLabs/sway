contract;

use std::{
    bytes::Bytes,
    hash::{
        Hash,
        sha256,
    },
    storage::{
        storage_bytes::*,
        storage_string::*,
        storage_vec::*,
    },
    string::String,
};

// ANCHOR: nested_storage_declaration
storage {
    nested_map_vec: StorageMap<u64, StorageVec<u8>> = StorageMap {},
    nested_map_string: StorageMap<u64, StorageString> = StorageMap {},
    nested_vec_bytes: StorageVec<StorageBytes> = StorageVec {},
}
// ANCHOR_END: nested_storage_declaration

abi StorageExample {
    #[storage(write)]
    fn store_map_vec();
    #[storage(read, write)]
    fn get_map_vec();
    #[storage(write)]
    fn store_map_string();
    #[storage(read)]
    fn get_map_string();
    #[storage(write)]
    fn store_vec();
    #[storage(read, write)]
    fn get_vec();
}

impl StorageExample for Contract {
    #[storage(write)]
    fn store_map_vec() {
        // ANCHOR: nested_vec_storage_write
        // Setup and initialize storage for the StorageVec.
        storage.nested_map_vec.try_insert(10, StorageVec {});

        // Method 1: Push to the vec directly
        storage.nested_map_vec.get(10).push(1u8);
        storage.nested_map_vec.get(10).push(2u8);
        storage.nested_map_vec.get(10).push(3u8);

        // Method 2: First get the storage key and then push the values.
        let storage_key_vec: StorageKey<StorageVec<u8>> = storage.nested_map_vec.get(10);
        storage_key_vec.push(4u8);
        storage_key_vec.push(5u8);
        storage_key_vec.push(6u8);
        // ANCHOR_END: nested_vec_storage_write
    }
    #[storage(read, write)]
    fn get_map_vec() {
        // ANCHOR: nested_vec_storage_read
        // Method 1: Access the StorageVec directly.
        let stored_val1: u8 = storage.nested_map_vec.get(10).pop().unwrap();
        let stored_val2: u8 = storage.nested_map_vec.get(10).pop().unwrap();
        let stored_val3: u8 = storage.nested_map_vec.get(10).pop().unwrap();

        // Method 2: First get the storage key and then access the value.
        let storage_key: StorageKey<StorageVec<u8>> = storage.nested_map_vec.get(10);
        let stored_val4: u8 = storage_key.pop().unwrap();
        let stored_val5: u8 = storage_key.pop().unwrap();
        let stored_val6: u8 = storage_key.pop().unwrap();
        // ANCHOR_END: nested_vec_storage_read
    }

    #[storage(write)]
    fn store_map_string() {
        // ANCHOR: nested_string_storage_write
        // Setup and initialize storage for the StorageString.
        storage.nested_map_string.try_insert(10, StorageString {});

        // Method 1: Store the string directly.
        let my_string = String::from_ascii_str("Fuel is blazingly fast");
        storage.nested_map_string.get(10).write_slice(my_string);

        // Method 2: First get the storage key and then write the value.
        let my_string = String::from_ascii_str("Fuel is modular");
        let storage_key: StorageKey<StorageString> = storage.nested_map_string.get(10);
        storage_key.write_slice(my_string);
        // ANCHOR_END: nested_string_storage_write
    }
    #[storage(read)]
    fn get_map_string() {
        // ANCHOR: nested_string_storage_read
        // Method 1: Access the string directly.
        let stored_string: String = storage.nested_map_string.get(10).read_slice().unwrap();

        // Method 2: First get the storage key and then access the value.
        let storage_key: StorageKey<StorageString> = storage.nested_map_string.get(10);
        let stored_string: String = storage_key.read_slice().unwrap();
        // ANCHOR_END: nested_string_storage_read
    }

    #[storage(write)]
    fn store_vec() {
        // ANCHOR: nested_vec_storage_write
        // Setup Bytes to store
        let mut my_bytes = Bytes::new();
        my_bytes.push(1u8);
        my_bytes.push(2u8);
        my_bytes.push(3u8);

        // Setup and initialize storage for the StorageBytes.
        storage.nested_vec_bytes.push(StorageBytes {});

        // Method 1: Store the bytes by accessing StorageBytes directly.
        storage
            .nested_vec_bytes
            .get(0)
            .unwrap()
            .write_slice(my_bytes);

        // Method 2: First get the storage key and then write the bytes.
        let storage_key: StorageKey<StorageBytes> = storage.nested_vec_bytes.get(0).unwrap();
        storage_key.write_slice(my_bytes);
        // ANCHOR_END: nested_vec_storage_write
    }
    #[storage(read, write)]
    fn get_vec() {
        // ANCHOR: nested_vec_storage_read
        // Method 1: Access the stored bytes directly.
        let stored_bytes: Bytes = storage.nested_vec_bytes.get(0).unwrap().read_slice().unwrap();

        // Method 2: First get the storage key and then access the stored bytes.
        let storage_key: StorageKey<StorageBytes> = storage.nested_vec_bytes.get(0).unwrap();
        let stored_bytes: Bytes = storage_key.read_slice().unwrap();
        // ANCHOR_END: nested_vec_storage_read
    }
}
