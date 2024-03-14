script;

use array_of_structs_abi::{Id, TestContract, Wrapper};
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xe2a4f86301f8b57ff2c93ce68366669fc2f0926dccd26f9f6550b049cb324a2c;
#[cfg(experimental_new_encoding = true)]
<<<<<<< HEAD
const CONTRACT_ID = 0xec759cace1887b3d0a7c38305cb72bef4a6799e0c503f04af587861603bb985f;
=======
const CONTRACT_ID = 0x78b2ec4ef197c4ebea87f4cb8c8cf46e1f54fa8896f387c8c3be66a7b8a74ed0;
>>>>>>> 5a1a9d79c (updating contract ids)

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
