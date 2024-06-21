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
    let contract_id = 0xe947c4b04b557d746b2f77988a4cd8528f36a71748c9bf830c0a8aad6e03191b;
    let caller = abi(MyContract, contract_id);

    let data = 1u64;
    let slice = raw_slice::from_parts::<u64>(__addr_of(&data), 1);
    let result = caller.test_function(slice);
    
    assert(result.len::<u8>() == slice.len::<u8>());
}
