contract;

// ANCHOR: import
use std::storage::storage_api::{read, write};
// ANCHOR_END: import
// ANCHOR: get
#[storage(read)]
fn get(key: b256) {
    // read::<T>(key, SLOT) where T = generic type
    let value = read::<u64>(key, 0);
}
// ANCHOR_END: get
// ANCHOR: store
#[storage(write)]
fn store(key: b256, value: u64) {
    // write(key, SLOT, T) where T = generic type
    write(key, 0, value);
}
// ANCHOR_END: store
