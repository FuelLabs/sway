script;

configurable {
    X: b256 = 0x0101010101010101010101010101010101010101010101010101010101010101,
}

configurable {
    Y: u64 = 42,
}

fn main() -> (b256, u64) {
    (X, Y)
}
