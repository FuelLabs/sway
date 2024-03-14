script;

use nested_struct_args_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x0fd8fed83ef774a35708706495b49f93254cc5ded343c3bd4416a70c8eb47e01;
#[cfg(experimental_new_encoding = true)]
<<<<<<< HEAD
const CONTRACT_ID = 0x0b25a96ec512963fdfe9928ef8e4c54c74e850f72cda5fbf6a82ad696a3edcb8;
=======
const CONTRACT_ID = 0x88b8410395d4014a9cec9d6544a97060cadbe5631907df9009a1e98cfc9283da;
>>>>>>> 5a1a9d79c (updating contract ids)

fn main() -> bool {
    let caller = abi(NestedStructArgs, CONTRACT_ID);

    let param_one = StructOne {
        inn: Inner { foo: 42 },
    };
    let param_two = StructTwo { foo: 42 };

    assert(caller.foo(param_one, param_two) == 85);
    true
}
