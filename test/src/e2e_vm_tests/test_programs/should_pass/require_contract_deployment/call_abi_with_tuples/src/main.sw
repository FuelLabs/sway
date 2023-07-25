script;

use abi_with_tuples::*;

fn main() -> bool {
    let the_abi = abi(MyContract, 0x7376aa3e846e8cefcd6e8c2dfea32e15cc8fef3bbdc67ef115897e0f09391cfc);

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
