contract;

// ANCHOR: initialization
use std::storage::storage_vec::*;

storage {
    // T = u64
    balance: StorageVec<u64> = StorageVec {},
    // T = (Identity, u64)
    user: StorageVec<(Identity, u64)> = StorageVec {},
}
// ANCHOR_END: initialization
// ANCHOR: reading_from_storage
#[storage(read)]
fn reading_from_storage(id: u64) {
    let balance = storage.balance.get(id).unwrap();

    let (user, value) = storage.user.get(id).unwrap().read();
}
// ANCHOR_END: reading_from_storage
// ANCHOR: writing_to_storage
#[storage(read, write)]
fn writing_to_storage(id: u64) {
    storage.user.push((msg_sender().unwrap(), id));
}
// ANCHOR_END: writing_to_storage
