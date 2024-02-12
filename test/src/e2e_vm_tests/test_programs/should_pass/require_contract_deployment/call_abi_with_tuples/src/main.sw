script;

use abi_with_tuples::*;

fn main() -> bool {
    let the_abi = abi(MyContract, 0x0bf9df7ec961330f61e9a13867e65854a6ac0f5514c220ef21da3c4885b4fccb);

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
