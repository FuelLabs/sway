script;

use array_of_structs_abi::{Id, TestContract, Wrapper};
use std::hash::*;

fn main() -> u64 {
    let addr = abi(TestContract, 0xbd1e3ad7022f6c170c6fb3643a1a0c4ad0f666a5a1d735b11255dbfff74e5a05);

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
