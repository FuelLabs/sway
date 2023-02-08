contract;

dep test_mod;
dep deep_mod;

use test_mod::A;
use deep_mod::deeper_mod::deep_fun as dfun;
use std::constants::{self, ZERO_B256};

pub fn fun() {
    let _ = std::option::Option::None;
    let _ = Option::None;
    let _ = std::vm::evm::evm_address::EvmAddress {
        value: b256::min(),
    };

    test_mod::test_fun();
    deep_mod::deeper_mod::deep_fun();
    std::assert::assert(true);
    let _ = core::primitives::u64::min();

    A::fun();
    test_mod::A::fun();

    let _ = std::constants::ZERO_B256;
    let _ = core::primitives::b256::min();
}
