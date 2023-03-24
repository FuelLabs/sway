script;

use nested_struct_args_abi::*;

fn main() -> bool {
    let contract_id = 0xf17db9ebfbf5470fb3955d6b86c038658e4a6016a28f8c1d64957fdee891001b;
    let caller = abi(NestedStructArgs, contract_id);

    let param_one = StructOne {
        inn: Inner { foo: 42 },
    };
    let param_two = StructTwo { foo: 42 };

    assert(caller.foo(param_one, param_two) == 85);
    true
}
