script;

use abi_with_tuples::*;
use std::assert::assert;

fn main() -> bool {
    let the_abi = abi(MyContract, 0xf403dd5c8c89bf7202bf25d48a381f9d1755b32cd128c3053ef60435a4999bd7);

    let param1 = (
        Person {
            age: 30
        },
        2u64,
    );
    let foo = the_abi.bug1(param1);
    assert(foo);

    let param2 = (
        Location::Earth(()),
        3u64
    );
    let bar = the_abi.bug2(param2);
    assert(bar);

    true
}
