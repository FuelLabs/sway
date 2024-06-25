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
    let contract_id = 0x3a9a75a3dc04f160ec6e286144eeffb6dd73fc0b4235b1793aa656be02e69d9b; // AUTO-CONTRACT-ID .
    let caller = abi(MyContract, contract_id);

    let data = 1u64;
    let slice = raw_slice::from_parts::<u64>(__addr_of(&data), 1);
    let result = caller.test_function(slice);
    
    assert(result.len::<u8>() == slice.len::<u8>());
}
