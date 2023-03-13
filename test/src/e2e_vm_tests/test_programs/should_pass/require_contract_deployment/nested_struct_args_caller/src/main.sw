script;

use nested_struct_args_abi::*;

fn main() -> bool {
    let contract_id = 0x7ab49d81780fa7ab8f7317a13039c796766b6868b670bce3fb5070033f610cbb;
    let caller = abi(NestedStructArgs, contract_id);

    let param_one = StructOne {
        inn: Inner { foo: 42 },
    };
    let param_two = StructTwo { foo: 42 };

    assert(caller.foo(param_one, param_two) == 85);
    true
}
