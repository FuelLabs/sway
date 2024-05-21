script;

use nested_struct_args_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x64390eb0cac08d41b6476ad57d711b88846ea35ac800d4fc3c95a551e4039432;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0xf12a88781f67f3ce2cdb4994e3571a6b42176c9ce63e3fdeb0247b1f319a8d83;

fn main() -> bool {
    let caller = abi(NestedStructArgs, CONTRACT_ID);

    let param_one = StructOne {
        inn: Inner { foo: 42 },
    };
    let param_two = StructTwo { foo: 42 };

    assert(caller.foo(param_one, param_two) == 85);
    true
}
