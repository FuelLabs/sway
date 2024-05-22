contract;

use std::{
    hash::*, 
    logging::log,
    string::String,
    storage::storage_vec::*,
    storage::storage_string::*, 
};

abi StorageVecOfStorageString {
    #[storage(read)] 
    fn count() -> u64;

    #[storage(read)] 
    fn get(index: u64) -> String;

    #[storage(read, write)] 
    fn push(text: String);

    #[storage(read, write)] 
    fn insert(text: String);
}


storage {
    texts: StorageVec<StorageString> = StorageVec {},
}

impl StorageVecOfStorageString for Contract {
   
    #[storage(read)] 
    fn count() -> u64 {
        storage.texts.len()
    }

    #[storage(read)] 
    fn get(index: u64) -> String
    {
        storage.texts.get(index).unwrap().read_slice().unwrap()
    }

    #[storage(read, write)] 
    fn push(text: String) {
        storage.texts.push(StorageString {});
        let index = storage.texts.len() - 1;
        storage.texts.get(index).unwrap().write_slice(text);
    }

    #[storage(read, write)] 
    fn insert(text: String) {
        storage.texts.insert(0, StorageString {});
        storage.texts.get(0).unwrap().write_slice(text);
    }
}