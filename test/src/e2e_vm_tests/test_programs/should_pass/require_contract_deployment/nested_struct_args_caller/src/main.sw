script;

use nested_struct_args_abi::*;

fn main() -> bool {
    let contract_id = 0xfa4bb608c7de0db473862926816eb23d17469ec2ef08685aab3c4ddd1892f9a8;
    let caller = abi(NestedStructArgs, contract_id);

    let param_one = StructOne {
        inn: Inner { foo: 42 },
    };
    let param_two = StructTwo { foo: 42 };

    assert(caller.foo(param_one, param_two) == 85);
    true
}
