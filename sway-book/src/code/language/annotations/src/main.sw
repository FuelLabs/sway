contract;

storage {
    var: u64 = 0,
}

// ANCHOR: read
#[storage(read)]
// ANCHOR_END: read
fn read() {
    let variable = storage.var;
}

// ANCHOR: write
#[storage(write)]
// ANCHOR_END: write
fn write() {
    storage.var += 1;
}

// ANCHOR: read_write
#[storage(read, write)]
// ANCHOR_END: read_write
fn read_write() {
    let var = storage.var;
    storage.var += 1;
}

fn example() {
    // ANCHOR: example
    let bar: str[4] = "sway";
    let baz: bool = true;
    // ANCHOR_END: example
}
