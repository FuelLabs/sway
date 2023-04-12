script;

fn main() -> bool {
    // A mutable variable with a non-zero initializer.
    let mut val_c = 0x0000000000000000000000000000000000000000000000000000000011111111;

    // By being marked as mutable it must be on the stack and not just the data section, but stack
    // variables still need to be initialized.
    assert(val_c == 0x0000000000000000000000000000000000000000000000000000000011111111);

    true
}
