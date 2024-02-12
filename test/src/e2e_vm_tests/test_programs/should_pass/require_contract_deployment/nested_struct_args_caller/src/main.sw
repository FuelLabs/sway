script;

use nested_struct_args_abi::*;

fn main() -> bool {
    let contract_id = 0xe36b6a1a6678bf99ff8d70868061e5c52413282a801a950ad8c3c6391f9bc305;
    let caller = abi(NestedStructArgs, contract_id);

    let param_one = StructOne {
        inn: Inner { foo: 42 },
    };
    let param_two = StructTwo { foo: 42 };

    assert(caller.foo(param_one, param_two) == 85);
    true
}
