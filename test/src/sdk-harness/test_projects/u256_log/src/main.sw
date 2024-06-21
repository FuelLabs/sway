contract;

abi MyContract {
    fn u256_log(x: u256, base: u256) -> u256;
}

impl MyContract for Contract {
    fn u256_log(x: u256, base: u256) -> u256 {
        x.log(base)
    }
}
