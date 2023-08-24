script;

use array_of_structs_abi::{Id, TestContract, Wrapper};
use std::hash::*;

fn sha256_str<T>(s: T) -> b256 {
    let mut hasher = Hasher::new();
    hasher.write_str(s);
    hasher.sha256()
}

fn main() -> u64 {
    let addr = abi(TestContract, 0x8be98f018738eb6e554372cc3e57c24475662e6eeff9781be50b146c41e72d05);

    let input = [Wrapper {
        id: Id {
            number: 42,
        },
    },
    Wrapper {
        id: Id {
            number: 66,
        },
    },
    ];

    let result = addr.return_array_of_structs(input);
    assert(result[0].id.number == 42);
    assert(result[1].id.number == 66);

    let result = addr.return_element_of_array_of_structs(input);
    assert(result.id.number == 42);

    let result = addr.return_element_of_array_of_strings([ "111", "222", "333"]);
    assert(sha256_str("111") == sha256_str(result));

    1
}
