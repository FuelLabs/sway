script;

use nested_struct_args_abi::*;
use std::assert::assert;

fn main() -> bool {
    let contract_id = 0x8f42239954e490e48bfbc22aade19f5169274fcbc4ffb2d6bfb7330954b73bd0;
    let caller = abi(NestedStructArgs, contract_id);

    let param_one = StructOne {
        inn: Inner { foo: 42 },
    };
    let param_two = StructTwo { foo: 42 };

    assert(caller.foo(param_one, param_two) == 85);
    true
}
