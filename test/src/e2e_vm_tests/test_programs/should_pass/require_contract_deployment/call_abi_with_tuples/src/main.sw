script;

use abi_with_tuples::{MyContract, Location, Person};


#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x5175a6a984a6d8f92622afd3d987f5b778f5741c56d55ee5993cc368b9afee10;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x2435d33fbcdcaea63ea81348866a3d565a84cc90f68a18513b0cef78ee3faa03;

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

    // We should be able to call functions on the return type.
    a.0.g();

    // But we should not be able to reference the type name,
    // because it is not bound.
    // let a = SomeStruct { data: 2 }; // This will fail

    true
}
