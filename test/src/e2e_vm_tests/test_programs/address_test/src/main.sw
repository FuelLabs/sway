script;

use std::address::Address;
use std::chain::assert;

fn main() -> bool {
    let bits = 0x8900c5bec4ca97d4febf9ceb4754a60d782abbf3cd815836c1872116f203f861;

    // test from()
    let addr = ~Address::from(bits);
    assert(addr.value == bits);

    // test into()
    let new_bits = addr.into();
    assert(new_bits == bits);

    true
}
