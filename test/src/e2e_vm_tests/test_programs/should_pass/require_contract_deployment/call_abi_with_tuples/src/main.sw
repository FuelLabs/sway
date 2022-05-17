script;

use abi_with_tuples::*;
use std::assert::assert;

fn main() -> bool {
    let the_abi = abi(MyContract, 0x0e1d75ab5fb3354cc69e3cbdef35e9a94c261df54671d78ce3f08bd66562c309);
    
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
