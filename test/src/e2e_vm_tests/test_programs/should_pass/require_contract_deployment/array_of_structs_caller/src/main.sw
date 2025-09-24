script;

use array_of_structs_abi::{Id, TestContract, Wrapper};
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x14ed3cd06c2947248f69d54bfa681fe40d26267be84df7e19e253622b7921bbe;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x9e0048a62c58414e5bbe689ad3fc0ef7f887bcf5668f01b5a60d513248d816c6; // AUTO-CONTRACT-ID ../../test_contracts/array_of_structs_contract --release

fn get_address() -> Option<std::address::Address> {
    Some(CONTRACT_ID.into())
}

fn main() -> u64 {
    // Test address being a complex expression
    let addr = abi(TestContract, get_address().unwrap().into());

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
