script;

use std::address::Address;
use std::assert::assert;

fn main() -> bool {
    let bits = 0x8900c5bec4ca97d4febf9ceb4754a60d782abbf3cd815836c1872116f203f861;

    let bits3 = 0x1000000000000000_0000000000000000_0000000000000000_0000000000000000;
    let bits4 = 0x1000000000000000_0000000000000000_00000000000000000_000000000000000;

    // test from()
    let addr = Address::from(bits);
    assert(addr.value == bits);

    // test into()
    let new_bits = addr.into();
    assert(new_bits == bits);

    // test Eq
    let addr1 = Address::from(0x0000000000000000_0000000000000000_0000000000000000_0000000000000000);
    let addr2 = Address::from(0x0000000000000000_0000000000000000_0000000000000000_0000000000000000);
    let addr3 = Address::from(0x1000000000000000_0000000000000000_0000000000000000_0000000000000000);
    let addr4 = Address::from(0x1000000000000000_0000000000000000_0000000000000000_0000000000000000);
    let addr5 = Address::from(0x0000000000000000_1000000000000000_0000000000000000_0000000000000000);
    let addr6 = Address::from(0x0000000000000000_1000000000000000_0000000000000000_0000000000000000);
    let addr7 = Address::from(0x0000000000000000_0000000000000000_1000000000000000_0000000000000000);
    let addr8 = Address::from(0x0000000000000000_0000000000000000_1000000000000000_0000000000000000);
    let addr9 = Address::from(0x0000000000000000_0000000000000000_0000000000000000_1000000000000000);
    let addrA = Address::from(0x0000000000000000_0000000000000000_0000000000000000_1000000000000000);
    let addrB = Address::from(0x0000000000000000_0000000000000000_0000000000000000_0000000000000001);
    let addrC = Address::from(0x0000000000000000_0000000000000000_0000000000000000_0000000000000001);
    let addrD = Address::from(0x0000001000000000_0010000000000000_0000000001000000_0000000100000000);
    let addrE = Address::from(0x0000001000000000_0010000000000000_0000000001000000_0000000100000000);

    assert(addr1 == addr2);
    assert(addr3 == addr4);
    assert(addr5 == addr6);
    assert(addr7 == addr8);
    assert(addr9 == addrA);
    assert(addrB == addrC);
    assert(addrD == addrE);

    assert(!(addr1 == addr3));
    assert(!(addr1 == addr5));
    assert(!(addr1 == addr7));
    assert(!(addr1 == addr9));
    assert(!(addr1 == addrB));
    assert(!(addr1 == addrD));

    assert(!(addr3 == addr5));
    assert(!(addr3 == addr7));
    assert(!(addr3 == addr9));
    assert(!(addr3 == addrB));
    assert(!(addr3 == addrD));

    assert(!(addr5 == addr7));
    assert(!(addr5 == addr9));
    assert(!(addr5 == addrB));
    assert(!(addr5 == addrD));

    assert(!(addr7 == addr9));
    assert(!(addr7 == addrB));
    assert(!(addr7 == addrD));

    assert(!(addr9 == addrB));
    assert(!(addr9 == addrD));

    true
}
