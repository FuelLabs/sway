script;
use array_of_structs_abi::{Id, TestContract, Wrapper};
use std::assert::assert;

fn main() -> u64 {
    let addr = abi(TestContract, 0x30ab89dada7ff41b1139f3bfa373e88261ca9829823c02423b6fbdcc2d8a1b8b);

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
