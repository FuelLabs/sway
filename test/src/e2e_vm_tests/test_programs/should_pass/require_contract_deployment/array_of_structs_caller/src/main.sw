script;
use array_of_structs_abi::{Id, TestContract, Wrapper};
use std::assert::assert;

fn main() -> u64 {
    let addr = abi(TestContract, 0xc17fec138a64fc2ebac467ff59979cb23179a83d7574a087327af490e415526e);

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

    1
}
