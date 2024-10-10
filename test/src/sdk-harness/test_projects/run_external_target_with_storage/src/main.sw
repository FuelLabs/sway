contract;

use std::constants::ZERO_B256;

storage {
    owner: Identity = Identity::Address(Address::from(ZERO_B256)),
    simple_value: u64 = 0,
}

abi RunExternalTest {
    fn double_value(foo: u64) -> u64;
    fn large_value() -> b256;
}

impl RunExternalTest for Contract {
    fn double_value(foo: u64) -> u64 {
        __log(2);
        foo * 2
    }
    fn large_value() -> b256 {
       0x00000000000000000000000059F2f1fCfE2474fD5F0b9BA1E73ca90b143Eb8d0
    }
}

// ANCHOR: fallback
#[fallback, storage(read, write)]
fn fallback() -> u64 {
    let iden = storage.owner.read();

    use std::call_frames::*;
    __log(3);
    __log(called_method());
    __log("double_value");
    __log(called_method() == "double_value");
    let foo = called_args::<u64>();
    foo * 3
}
// ANCHOR_END: fallback
