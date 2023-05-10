script;

use nested_struct_args_abi::*;

fn main() -> bool {
    let contract_id = 0x1ce4c30fc1fe5d4639683ad1330de7739efd297671a540ffe9012e9cf3953fe7;
    let caller = abi(NestedStructArgs, contract_id);

    let param_one = StructOne {
        inn: Inner { foo: 42 },
    };
    let param_two = StructTwo { foo: 42 };

    assert(caller.foo(param_one, param_two) == 85);
    true
}
