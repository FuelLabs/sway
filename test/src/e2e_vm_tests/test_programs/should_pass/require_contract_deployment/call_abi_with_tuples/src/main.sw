script;

use abi_with_tuples::*;

fn main() -> bool {
    let the_abi = abi(MyContract, 0x6865d45808bc8127ecfaf6ad4376175cd9d9cdddcc10e9aa3a62228e690e540e);

    let param1 = (
        Person {
            age: 30
        },
        2u64,
    );
    let foo = the_abi.bug1(param1);
    assert(foo);

    let param2 = (
        Location::Earth,
        3u64
    );
    let bar = the_abi.bug2(param2);
    assert(bar);

    true
}
