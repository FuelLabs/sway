script;

use abi_with_tuples::*;

fn main() -> bool {
    let the_abi = abi(MyContract, 0xca51db1c6f5cb1c694f1358af9130ed1167738f518a7ce49f6936443f66295f2);

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
