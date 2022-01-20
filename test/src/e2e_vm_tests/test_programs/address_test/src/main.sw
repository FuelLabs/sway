script;

use std::address::Address;
use std::chain::assert;

fn main() -> bool {
    let bits = 0x8900c5bec4ca97d4febf9ceb4754a60d782abbf3cd815836c1872116f203f861;
    let bits1 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let bits2 = 0x0000000000000000000000000000000000000000000000000000000000000000;

    // test from()
    let addr = ~Address::from(bits);
    assert(addr.value == bits);

    // test into()
    let new_bits = addr.into();
    assert(new_bits == bits);

    // test Ord
    let addr1 = ~Address::from(bits1);
    let addr2 = ~Address::from(bits2);
    assert(addr1 == addr2); // this fails atm.

    true
}
