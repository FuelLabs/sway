contract;

use std::storage::storage_bytes::*;
use std::bytes::Bytes;

storage {
    bytes: StorageBytes = StorageBytes {},
}

abi StorageBytesTest {
    #[storage(read, write)]
    fn store_bytes(vec: Vec<u8>);
    #[storage(read)]
    fn assert_stored_bytes(vec: Vec<u8>);
    #[storage(read, write)]
    fn clear_stored_bytes() -> bool;
    #[storage(read)]
    fn len() -> u64;
}

impl StorageBytesTest for Contract {
    #[storage(read, write)]
    fn store_bytes(vec: Vec<u8>) {
        let mut vec = vec;
        let bytes = Bytes::from_vec_u8(vec);

        storage.bytes.write_slice(bytes);
    }

    #[storage(read)]
    fn assert_stored_bytes(vec: Vec<u8>) {
        let mut vec = vec;
        let bytes = Bytes::from_vec_u8(vec);
        let stored_bytes = storage.bytes.read_slice().unwrap();

        assert(bytes.len() == stored_bytes.len());
        assert(bytes == stored_bytes);
    }

    #[storage(read, write)]
    fn clear_stored_bytes() -> bool {
        let cleared = storage.bytes.clear();

        assert(storage.bytes.len() == 0);
        assert(storage.bytes.read_slice().is_none());

        cleared
    }

    #[storage(read)]
    fn len() -> u64 {
        storage.bytes.len()
    }
}
