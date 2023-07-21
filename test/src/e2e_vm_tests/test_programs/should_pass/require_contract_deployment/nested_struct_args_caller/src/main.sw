script;

use nested_struct_args_abi::*;

fn main() -> bool {
    let contract_id = 0x33639d15e9187676324cf8a42f6e15349660dc04d4cd52a9919bb9b3f15f732e;
    let caller = abi(NestedStructArgs, contract_id);

    let param_one = StructOne {
        inn: Inner { foo: 42 },
    };
    let param_two = StructTwo { foo: 42 };

    assert(caller.foo(param_one, param_two) == 85);
    true
}
