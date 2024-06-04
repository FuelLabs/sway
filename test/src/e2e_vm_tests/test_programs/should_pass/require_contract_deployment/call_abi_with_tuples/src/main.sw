script;

use abi_with_tuples::{MyContract, Location, Person};

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xb351aff8258ce46d16a71be666dd2b0b09d72243105c51f4423765824e59cac9;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x17d3f0c7895e749e9aea69b1b1c3b3774e8a84add7275de61065dfa3b9785dee;

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

    // This fn returns some_module::SomeStruct, and this struct
    // should not be manually imported
    // We want the compiler to import its AbiDecode impl automatically
    let a = the_abi.struct_at_return();
    assert(a.0.data == 1);

    true
}
