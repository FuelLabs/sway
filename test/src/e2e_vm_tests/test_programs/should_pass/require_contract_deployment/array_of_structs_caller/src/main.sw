script;

use array_of_structs_abi::{Id, TestContract, Wrapper};
use std::hash::sha256;

fn main() -> u64 {
    let addr = abi(TestContract, 0x03d3ef50c3cf3716962cd0a447c68c3f2c85b980425e4b313c275dd1da28de8b);

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
    assert(sha256("111") == sha256(result));

    1
}
