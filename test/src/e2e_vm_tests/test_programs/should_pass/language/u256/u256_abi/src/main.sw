script;
configurable {
    SOME_U256: u256 = 0x00000000000000000000000000000000000000000000000000000001u256,
}

fn main() -> u256 {
    log(0x00000000000000000000000000000000000000000000000000000002u256);
    SOME_U256
}
