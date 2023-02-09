contract;

use std::storage::StorageBytes;
use std::bytes::Bytes;

storage {
    bytes: StorageBytes = StorageBytes {},
}

abi StorageBytesTest {
    #[storage(write)]
    fn store_bytes(vec: Vec<u8>);
    #[storage(read)]
    fn assert_stored_bytes(vec: Vec<u8>);
    #[storage(read)]
    fn len() -> u64;
}

impl StorageBytesTest for Contract {
    #[storage(write)]
    fn store_bytes(vec: Vec<u8>) {
        let mut vec = vec;
        let bytes = Bytes::from_vec_u8(vec);

        storage.bytes.store_bytes(bytes);
    }

    #[storage(read)]
    fn assert_stored_bytes(vec: Vec<u8>) {
        let mut vec = vec;
        let bytes = Bytes::from_vec_u8(vec);
        let stored_bytes = storage.bytes.into_bytes().unwrap();

        assert(bytes.len() == stored_bytes.len());
        assert(bytes == stored_bytes);
    }

    #[storage(read)]
    fn len() -> u64 {
        let bytes = storage.bytes.into_bytes();
        match bytes {
            Option::Some(bytes) => {
                bytes.len()
            }
            Option::None => {
                0
            }
        }
    }
}
