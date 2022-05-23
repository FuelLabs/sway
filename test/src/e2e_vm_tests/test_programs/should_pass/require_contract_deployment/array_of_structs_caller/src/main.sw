script;
use array_of_structs_abi::{Id, TestContract, Wrapper};
use std::assert::assert;

fn main() -> u64 {
    let addr = abi(TestContract, 0xf1221ef2f1b9bb5443279c25cee9337bb3cecfbfb24427ad629f2e4ebca658da);

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
