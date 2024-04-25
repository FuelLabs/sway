// target-fuelvm

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

// ::check-ir::

// check: contract {
// check: fn get_b256<42123b96>(val: ptr b256, __ret_value: ptr b256) -> ptr b256,
// check: fn get_s<fc62d029>(val1 !2: u64, val2: ptr b256, __ret_value: ptr { u64, b256 }) -> ptr { u64, b256 }
// check: fn get_u64<9890aef4>(val !5: u64) -> u64
