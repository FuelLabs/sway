script;

use nested_struct_args_abi::*;

fn main() -> bool {
    let contract_id = 0xe4eb85ab28ca132848eca751949ba4c060a2ca757e87ef757baf2bccd4086437;
    let caller = abi(NestedStructArgs, contract_id);

    let param_one = StructOne {
        inn: Inner { foo: 42 },
    };
    let param_two = StructTwo { foo: 42 };

    assert(caller.foo(param_one, param_two) == 85);
    true
}
