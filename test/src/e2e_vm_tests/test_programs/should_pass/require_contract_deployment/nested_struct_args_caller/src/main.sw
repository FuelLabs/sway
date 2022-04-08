script;

use nested_struct_args_abi::*;
use std::assert::assert;

fn main() -> bool {
    let contract_id = 0xd7d2c1536ae34f2fea108e6c303f72d7fe96ab91e0f206b055fd734bc27e3608;
    let caller = abi(NestedStructArgs, contract_id);

    let param_one = StructOne { inn: Inner { foo : 42 } };
    let param_two = StructTwo { foo: 42 };

    assert(caller.foo(param_one, param_two) == 85);
    true
}
