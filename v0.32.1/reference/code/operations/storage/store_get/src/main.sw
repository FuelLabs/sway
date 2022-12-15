contract;

// ANCHOR: import
use std::storage::{get, store};
// ANCHOR_END: import
// ANCHOR: get
#[storage(read)]
fn read(key: b256) {
    // get::<T>(key) where T = generic type
    let value = get::<u64>(key);
}
// ANCHOR_END: get
// ANCHOR: store
#[storage(write)]
fn write(key: b256, value: u64) {
    // store(key, T) where T = generic type
    store(key, value);
}
// ANCHOR_END: store
