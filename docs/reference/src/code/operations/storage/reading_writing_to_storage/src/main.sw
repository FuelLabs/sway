contract;

// ANCHOR: declaration
storage {
    counter: u64 = 0,
}
// ANCHOR_END: declaration
// ANCHOR: read
#[storage(read)]
fn read() {
    let counter = storage.counter;
}
// ANCHOR_END: read
// ANCHOR: write
#[storage(write)]
fn write() {
    storage.counter += 1;
}
// ANCHOR_END: write
// ANCHOR: read_write
#[storage(read, write)]
fn read_write() {
    let counter = storage.counter;
    storage.counter += 1;
}
// ANCHOR_END: read_write
