contract;

use std::hash::*;

// ANCHOR: initialization

storage {
    // k = Identity, v = u64
    balance: StorageMap<Identity, u64> = StorageMap::<Identity, u64> {},
    // k = (Identity, u64), v = bool
    user: StorageMap<(Identity, u64), bool> = StorageMap::<(Identity, u64), bool> {},
}
// ANCHOR_END: initialization
// ANCHOR: reading_from_storage
#[storage(read)]
fn reading_from_storage(id: u64) {
    let user = storage.user.get((msg_sender().unwrap(), id)).read();
}
// ANCHOR_END: reading_from_storage
// ANCHOR: writing_to_storage
#[storage(read, write)]
fn writing_to_storage() {
    let balance = storage.balance.get(msg_sender().unwrap()).read();
    storage.balance.insert(msg_sender().unwrap(), balance + 1);
}
// ANCHOR_END: writing_to_storage
