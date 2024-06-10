script;

use nested_struct_args_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x64390eb0cac08d41b6476ad57d711b88846ea35ac800d4fc3c95a551e4039432;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x1b9c368c7192e419e4b27cd8ff0d4231b913a4fe056f080c90a691f5eb53cc63;

fn main() -> bool {
    let caller = abi(NestedStructArgs, CONTRACT_ID);

    let param_one = StructOne {
        inn: Inner { foo: 42 },
    };
    let param_two = StructTwo { foo: 42 };

    assert(caller.foo(param_one, param_two) == 85);
    true
}
