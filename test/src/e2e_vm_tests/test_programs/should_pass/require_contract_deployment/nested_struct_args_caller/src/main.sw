script;

use nested_struct_args_abi::*;

fn main() -> bool {
    let contract_id = 0x0fd8fed83ef774a35708706495b49f93254cc5ded343c3bd4416a70c8eb47e01;
    let caller = abi(NestedStructArgs, contract_id);

    let param_one = StructOne {
        inn: Inner { foo: 42 },
    };
    let param_two = StructTwo { foo: 42 };

    assert(caller.foo(param_one, param_two) == 85);
    true
}
