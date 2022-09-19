script;

use abi_with_tuples::*;
use std::assert::assert;

fn main() -> bool {
    let the_abi = abi(MyContract, 0x32a5e5b389bda4bbf8edad1cdb3abe8e1e004bc947ebc6212e307ae7809b554f);

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
