contract;

// ANCHOR: initialization
use std::chain::auth::msg_sender;

storage {
    // k = Identity, v = u64
    balance: StorageMap<Identity, u64> = StorageMap {},
    // k = (Identity, u64), v = bool
    user: StorageMap<(Identity, u64), bool> = StorageMap {},
}
// ANCHOR_END: initialization
// ANCHOR: reading_from_storage
#[storage(read)]
fn reading_from_storage(id: u64) {
    let user = storage.user.get((msg_sender().unwrap(), id));
}
// ANCHOR_END: reading_from_storage
// ANCHOR: writing_to_storage
#[storage(read, write)]
fn writing_to_storage() {
    let balance = storage.balance.get(msg_sender().unwrap());
    storage.balance.insert(msg_sender().unwrap(), balance + 1);
}
// ANCHOR_END: writing_to_storage
