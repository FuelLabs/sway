script;

use abi_with_tuples::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xb351aff8258ce46d16a71be666dd2b0b09d72243105c51f4423765824e59cac9;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0xa72bdef88f26490082967d0bb6a51e1789d8f404ad95401ffe388a2c43c94fdc;

fn main() -> bool {
    let the_abi = abi(MyContract, CONTRACT_ID);

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
