contract;

abi ExternalTarget {
    fn double_input(value: u64) -> u64;
}

// ANCHOR: target
impl ExternalTarget for Contract {
    fn double_input(value: u64) -> u64 {
        value * 2
    }
}
// ANCHOR_END: target
