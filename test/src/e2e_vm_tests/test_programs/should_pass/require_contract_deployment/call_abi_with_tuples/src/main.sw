script;

use abi_with_tuples::{MyContract, Location, Person};


#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xfdc14550c8aee742cd556d0ab7f378b7be0d3b1e6e086c097352e94590d4ed02;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0xaead2479b811f477c799379f25b5102bb697105849365b8b616293fa0441585a; // AUTO-CONTRACT-ID ../../test_contracts/abi_with_tuples_contract --release

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
