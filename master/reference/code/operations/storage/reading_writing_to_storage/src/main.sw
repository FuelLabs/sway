contract;

// ANCHOR: declaration
storage {
    counter: u64 = 0,
}
// ANCHOR_END: declaration
// ANCHOR: read
#[storage(read)]
fn read() {
    let counter = storage.counter.read();
}
// ANCHOR_END: read
// ANCHOR: write
#[storage(write)]
fn write() {
    storage.counter.write(storage.counter.read() + 1);
}
// ANCHOR_END: write
// ANCHOR: read_write
#[storage(read, write)]
fn read_write() {
    let counter = storage.counter.read();
    storage.counter.write(counter + 1);
}
// ANCHOR_END: read_write
