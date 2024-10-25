script;

use nested_struct_args_abi::*;

#[cfg(experimental_encoding_v1 = false)]
const CONTRACT_ID = 0xe63d33a1b3a6903808b379f6a41a72fa8a370e8b76626775e7d9d2f9c4c5da40;
#[cfg(experimental_encoding_v1 = true)]
const CONTRACT_ID = 0xc26e4f54f9bca811be460d07aaedfa12d46477378573947ae95653affba962be; // AUTO-CONTRACT-ID ../../test_contracts/nested_struct_args_contract --release

fn main() -> bool {
    let caller = abi(NestedStructArgs, CONTRACT_ID);

    let param_one = StructOne {
        inn: Inner { foo: 42 },
    };
    let param_two = StructTwo { foo: 42 };

    assert(caller.foo(param_one, param_two) == 85);
    true
}
