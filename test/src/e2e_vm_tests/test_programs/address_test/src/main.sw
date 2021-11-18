script;

use std::address::Address;

    // @review it feels a bit clunky to have to refer to the inner value of an address type to get the value. Is there a better way this could be implemented? ie: when refering to an `Address`, always return the inner val by default...

fn main() -> bool {
    let bits = 0x8900c5bec4ca97d4febf9ceb4754a60d782abbf3cd815836c1872116f203f861;

    // test from()
    let addr = ~Address::from(bits);
    let t1 = addr.inner == bits;

    // test into()
    let new_bits = addr.into();
    let t2 = new_bits == bits;

    t1 && t2
}
