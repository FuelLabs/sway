script;

use nested_struct_args_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xc615be7b48402210cbec3bc1667ab5a8093d449d5d8d1fdcc26e6f18e7942ea9;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x9d00e98f60e2f09550861b1ddfbf26d999171696d161ba1df36fc2fd10e73097;

fn main() -> bool {
    let caller = abi(NestedStructArgs, CONTRACT_ID);

    let param_one = StructOne {
        inn: Inner { foo: 42 },
    };
    let param_two = StructTwo { foo: 42 };

    assert(caller.foo(param_one, param_two) == 85);
    true
}
