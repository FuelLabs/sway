script;

use abi_with_tuples::*;

fn main() -> bool {
    let the_abi = abi(MyContract, 0xb895678d87e2eb58a7ff5d2de7c6dedfa82d2af7c7629a8f8c862b92452f6e8c);

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
