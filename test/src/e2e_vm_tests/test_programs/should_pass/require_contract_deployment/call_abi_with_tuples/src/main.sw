script;

use abi_with_tuples::*;
use std::assert::assert;

fn main() -> bool {
    let the_abi = abi(MyContract, 0x417e8ee99a538fb03b032862bedf70ccd28dcec4a0fb455c72700f5234467f48);

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
