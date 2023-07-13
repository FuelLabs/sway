script;

use nested_struct_args_abi::*;

fn main() -> bool {
    let contract_id = 0xedc6a0a5dff075d2c760ceb94208d4ee3382ab7bb4193638073e43e8932f6a1f;
    let caller = abi(NestedStructArgs, contract_id);

    let param_one = StructOne {
        inn: Inner { foo: 42 },
    };
    let param_two = StructTwo { foo: 42 };

    assert(caller.foo(param_one, param_two) == 85);
    true
}
