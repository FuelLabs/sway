script;

use nested_struct_args_abi::*;
use std::assert::assert;

fn main() -> bool {
    let contract_id = 0xa29c8e30ab807e331a59e05d213ee7105d0703d95192226473919234f2905e81;
    let caller = abi(NestedStructArgs, contract_id);

    let param_one = StructOne { inn: Inner { foo : 42 } };
    let param_two = StructTwo { foo: 42 };

    assert(caller.foo(param_one, param_two) == 85);
    true
}
