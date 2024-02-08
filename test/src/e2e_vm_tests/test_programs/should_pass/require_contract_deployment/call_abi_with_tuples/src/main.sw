script;

use abi_with_tuples::*;

fn main() -> bool {
    let the_abi = abi(MyContract, 0xe507ae21649fbd2b48ccda116687d2ff164b190c09d33d9d480981323af16be7);

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
