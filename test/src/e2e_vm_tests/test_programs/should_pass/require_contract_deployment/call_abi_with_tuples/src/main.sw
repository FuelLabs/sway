script;

use abi_with_tuples::*;
use std::assert::assert;

fn main() -> bool {
    let the_abi = abi(MyContract, 0xeff8d28ce02d20aac8e32c811f1760f5031670d6a141bd7c0ee6ba594ac31355);
    
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
