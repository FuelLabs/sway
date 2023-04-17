script;

use nested_struct_args_abi::*;

fn main() -> bool {
    let contract_id = 0xc07c133be5867020f483c34e045c9162867d6b40accc022525e50c048d17d679;
    let caller = abi(NestedStructArgs, contract_id);

    let param_one = StructOne {
        inn: Inner { foo: 42 },
    };
    let param_two = StructTwo { foo: 42 };

    assert(caller.foo(param_one, param_two) == 85);
    true
}
