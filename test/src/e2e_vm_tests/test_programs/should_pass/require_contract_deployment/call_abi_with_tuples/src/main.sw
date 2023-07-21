script;

use abi_with_tuples::*;

fn main() -> bool {
    let the_abi = abi(MyContract, 0x553558ff0c47820e5012953153ec30658bd2df11ef013139839551579dbff2f0);

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
