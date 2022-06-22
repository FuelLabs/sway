contract;

use std::option::*;
use std::result::*;
use std::storage::{StorageVec, StorageVecError};

abi MyContract {
    #[storage(read, write)]
    fn vec_u16_push(value: u16);
}

storage {
    vec_u16: StorageVec<u8>,
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn vec_u16_push(value: u16) {
        storage.vec_u16.push(value);
    }
}
