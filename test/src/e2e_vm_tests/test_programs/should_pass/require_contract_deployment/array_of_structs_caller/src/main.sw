script;

use array_of_structs_abi::{Id, TestContract, Wrapper};
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x14ed3cd06c2947248f69d54bfa681fe40d26267be84df7e19e253622b7921bbe;
#[cfg(experimental_new_encoding = true)]
<<<<<<< HEAD
const CONTRACT_ID = 0x3a538fcd0aacef0147f0f673e1717a8c6756e2f091438a58b0795553e728b5be; // AUTO-CONTRACT-ID ../../test_contracts/array_of_structs_contract --release
=======
const CONTRACT_ID = 0x5ffc4edf4c66f00f3d4eafc2b74cd6ae6c6cb308a4a9e4db3ec1cd5e4a9c698b; // AUTO-CONTRACT-ID ../../test_contracts/array_of_structs_contract --release
>>>>>>> 15185e32f (update tests)

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
