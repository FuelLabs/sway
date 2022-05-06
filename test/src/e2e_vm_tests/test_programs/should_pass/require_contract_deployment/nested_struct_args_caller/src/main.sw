script;

use nested_struct_args_abi::*;
use std::assert::assert;

fn main() -> bool {
    let contract_id = 0x7d2a7d9a5cf7d86cad139bbdacc687329767479e25fb703581e8e697579fbb1e;
    let caller = abi(NestedStructArgs, contract_id);

    let param_one = StructOne { inn: Inner { foo : 42 } };
    let param_two = StructTwo { foo: 42 };

    assert(caller.foo(param_one, param_two) == 85);
    true
}
