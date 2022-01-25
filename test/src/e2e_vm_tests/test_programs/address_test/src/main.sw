script;

use std::address::Address;
use std::chain::assert;

fn main() -> bool {
    let bits = 0x8900c5bec4ca97d4febf9ceb4754a60d782abbf3cd815836c1872116f203f861;

    let bits3 = 0x1000000000000000000000000000000000000000000000000000000000000000;
    let bits4 = 0x1000000000000000000000000000000000000000000000000000000000000000;
// 0x 0000000000000000 0000000000000000 0000000000000000 0000000000000000
    // test from()
    let addr = ~Address::from(bits);
    assert(addr.value == bits);

    // test into()
    let new_bits = addr.into();
    assert(new_bits == bits);

    // test Ord
    let addr1 = ~Address::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let addr2 = ~Address::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let addr3 = ~Address::from(0x1000000000000000000000000000000000000000000000000000000000000000);
    let addr4 = ~Address::from(0x1000000000000000000000000000000000000000000000000000000000000000);
    let addr5 = ~Address::from(0x0000000000000000100000000000000000000000000000000000000000000000);
    let addr6 = ~Address::from(0x0000000000000000100000000000000000000000000000000000000000000000);
    let addr7 = ~Address::from(0x0000000000000000000000000000000010000000000000000000000000000000);
    let addr8 = ~Address::from(0x0000000000000000000000000000000010000000000000000000000000000000);
    let addr9 = ~Address::from(0x0000000000000000000000000000000000000000000000001000000000000000);
    let addr10 = ~Address::from(0x0000000000000000000000000000000000000000000000001000000000000000);
    let addr11 = ~Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let addr12 = ~Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let addr13 = ~Address::from(0x0000001000000000001000000000000000000000010000000000000100000000);
    let addr14 = ~Address::from(0x0000001000000000001000000000000000000000010000000000000100000000);

    assert(addr1 == addr2);
    assert(addr3 == addr4);
    assert(addr5 == addr6);
    assert(addr7 == addr8);
    assert(addr9 == addr10);
    assert(addr11 == addr12);
    assert(addr13 == addr14);

    assert(!(addr1 == addr3));
    assert(!(addr1 == addr5));
    assert(!(addr1 == addr7));
    assert(!(addr1 == addr9));
    assert(!(addr1 == addr11));
    assert(!(addr1 == addr13));

    assert(!(addr3 == addr5));
    assert(!(addr3 == addr7));
    assert(!(addr3 == addr9));
    assert(!(addr3 == addr11));
    assert(!(addr3 == addr13));

    assert(!(addr5 == addr7));
    assert(!(addr5 == addr9));
    assert(!(addr5 == addr11));
    assert(!(addr5 == addr13));

    assert(!(addr7 == addr9));
    assert(!(addr7 == addr11));
    assert(!(addr7 == addr13));

    assert(!(addr9 == addr11));
    assert(!(addr9 == addr13));

    true
}
