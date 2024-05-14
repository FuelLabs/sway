contract;

use std::{
    bytes::Bytes,
    hash::*,
    storage::storage_string::*,
    storage::storage_vec::*,
    string::String,
};

storage {
    msgs: StorageMap<b256, StorageString> = StorageMap::<b256, StorageString> {},
    msgs_sender: StorageMap<b256, Identity> = StorageMap::<b256, Identity> {},
    ids: StorageVec<b256> = StorageVec::<b256> {},
}

abi Thread {
    #[storage(read, write)]
    fn insert_msg(id: b256, msg: String) -> b256;

    #[storage(read)]
    fn get_ids() -> Vec<b256>;

    #[storage(read)]
    fn get_msg(id: b256) -> String;

    #[storage(read)]
    fn get_sender(id: b256) -> Address;
}

impl Thread for Contract {
    #[storage(read, write)]
    fn insert_msg(id: b256, msg: String) -> b256 {
        let key_of_string = storage.msgs.get(id);
        key_of_string.write_slice(msg);
        let sender = msg_sender().unwrap();
        storage.msgs_sender.insert(id, sender);
        storage.ids.push(id);
        id
    }

    #[storage(read)]
    fn get_ids() -> Vec<b256> {
        storage.ids.load_vec()
    }

    #[storage(read)]
    fn get_msg(id: b256) -> String {
        storage.msgs.get(id).read_slice().unwrap()
    }

    #[storage(read)]
    fn get_sender(id: b256) -> Address {
        let id = storage.msgs_sender.get(id).try_read().unwrap();
        id.as_address().unwrap()
    }
}