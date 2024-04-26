script;

use array_of_structs_abi::{Id, TestContract, Wrapper};
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x7fae96947a8cad59cc2a25239f9f80897955d4c1b10d31510681f15842b93265;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x991629307ab1ce17b2e7f7e0567343036862d4a6f675c317ec4d6bd0311e3426;

fn main() -> u64 {
    let addr = abi(TestContract, CONTRACT_ID);

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

    let result = addr.return_element_of_array_of_strings([ 
        __to_str_array("111"), 
        __to_str_array("222"), 
        __to_str_array("333")
    ]);
    assert(sha256("111") == sha256_str_array(result));

    1
}
