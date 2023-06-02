contract;

storage {
    var: u64 = 0,
}

// ANCHOR: read
#[storage(read)]
// ANCHOR_END: read
fn read() {
    let variable = storage.var.read();
}

// ANCHOR: write
#[storage(write)]
// ANCHOR_END: write
fn write() {
    storage.var.write(storage.var.read() + 1);
}

// ANCHOR: read_write
#[storage(read, write)]
// ANCHOR_END: read_write
fn read_write() {
    let var = storage.var.read();
    storage.var.write(var + 1);
}

fn example() {
    // ANCHOR: example
    let bar: str[4] = "sway";
    let baz: bool = true;
    // ANCHOR_END: example
}

abi MyContract {
    // ANCHOR: payable
    #[payable]
    fn deposit();
    // ANCHOR_END: payable
}
