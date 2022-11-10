script;

use nested_struct_args_abi::*;
use std::assert::assert;

fn main() -> bool {
    let contract_id = 0xfb0a1427ed3aa55c69d26d74d0ad065335ac57614d824587fee1009bfd3de70b;
    let caller = abi(NestedStructArgs, contract_id);

    let param_one = StructOne {
        inn: Inner { foo: 42 },
    };
    let param_two = StructTwo { foo: 42 };

    assert(caller.foo(param_one, param_two) == 85);
    true
}
