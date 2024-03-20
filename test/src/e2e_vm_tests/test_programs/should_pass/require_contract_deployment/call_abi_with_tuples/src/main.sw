script;

use abi_with_tuples::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x1200d031e9c10f8d9bd9dd556a98a0c88e74a4da991047556f78b1bcc1be2ab6;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x5de3f05e6429ea4b14d51b4797ba3f02a11cd0b546853d339f7b5ff13991c700;

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
