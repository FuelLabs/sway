script;

use abi_with_tuples::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x1200d031e9c10f8d9bd9dd556a98a0c88e74a4da991047556f78b1bcc1be2ab6;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x69e6f610ac1684cc754ed44fe67b3e9d3459bdb23e2db0956b57adb60f7658c7;

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
