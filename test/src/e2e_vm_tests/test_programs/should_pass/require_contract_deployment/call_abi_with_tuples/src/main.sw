script;

use abi_with_tuples::*;

fn main() -> bool {
    let the_abi = abi(MyContract, 0xae6af2e324ada9943f1c9b3d6ea2db1b9f9bb1cd1732321877b59bd75b6821fd);

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
