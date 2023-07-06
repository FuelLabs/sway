contract;

use std::bytes::Bytes;
use std::string::String;
use std::storage::storage_string::*;

storage {
    stored_string: StorageString = StorageString {},
}

abi MyContract {
    #[storage(read, write)]
    fn clear_string() -> bool;
    #[storage(read)]
    fn get_string() -> Bytes;
    #[storage(write)]
    fn store_string(string: String);
    #[storage(read)]
    fn stored_len() -> u64;
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn clear_string() -> bool {
        storage.stored_string.clear()
    }

    #[storage(read)]
    fn get_string() -> Bytes {
        match storage.stored_string.read_slice() {
            Option::Some(string) => {
                string.bytes
            },
            Option::None => Bytes::new(),
        }
    }

    #[storage(write)]
    fn store_string(string: String) {
        storage.stored_string.write_slice(string);
    }

    #[storage(read)]
    fn stored_len() -> u64 {
        storage.stored_string.len()
    }
}
