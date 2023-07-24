script;

use nested_struct_args_abi::*;

fn main() -> bool {
    let contract_id = 0xbc305fb5e79e14f1cbd578ec5661686610720b6fe3117ffd54f49ef00bd4b011;
    let caller = abi(NestedStructArgs, contract_id);

    let param_one = StructOne {
        inn: Inner { foo: 42 },
    };
    let param_two = StructTwo { foo: 42 };

    assert(caller.foo(param_one, param_two) == 85);
    true
}
