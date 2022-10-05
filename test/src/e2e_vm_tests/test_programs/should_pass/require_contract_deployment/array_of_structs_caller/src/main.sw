script;
use array_of_structs_abi::{Id, TestContract, Wrapper};
use std::{assert::assert, hash::sha256};

fn main() -> u64 {
    let addr = abi(TestContract, 0xda0aa9fce0e09044c49954b15d9243bedf40c97e8d64be5c36de4b2d25b1a43a);

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
