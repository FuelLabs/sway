contract;

abi MyContract {
    fn test_function(a: raw_slice) -> raw_slice;
}

impl MyContract for Contract {
    fn test_function(a: raw_slice) -> raw_slice {
        a
    }
}

#[test]
fn test_success() {
    let contract_id = 0xf1bb4dd787c32f29c2243fdffc924858e7a2ec6d215397c98f88ca21c1bb8f6c;
    let caller = abi(MyContract, contract_id);

    let data = 1u64;
    let slice = raw_slice::from_parts::<u64>(__addr_of(&data), 1);
    let result = caller.test_function(slice);
    
    assert(result.len::<u8>() == slice.len::<u8>());
}
