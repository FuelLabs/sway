contract;

struct S {
    x: u64,
    y: b256,
}

abi Test {
    fn get_u64(val: u64) -> u64;
    fn get_b256(val: b256) -> b256;
    fn get_s(val1: u64, val2: b256) -> S;
}

impl Test for Contract {
    fn get_u64(val: u64) -> u64 {
        val
    }

    fn get_b256(val: b256) -> b256 {
        val
    }

    fn get_s(val1: u64, val2: b256) -> S {
        S {
            x: val1,
            y: val2,
        }
    }
}
