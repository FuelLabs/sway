script;

use nested_struct_args_abi::*;

fn main() -> bool {
    let contract_id = 0xa16f173a7e49805781eeb4b797fad3debb42a443c4208555904a106b43a75368;
    let caller = abi(NestedStructArgs, contract_id);

    let param_one = StructOne {
        inn: Inner { foo: 42 },
    };
    let param_two = StructTwo { foo: 42 };

    assert(caller.foo(param_one, param_two) == 85);
    true
}
