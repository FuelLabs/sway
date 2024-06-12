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
    let contract_id = 0x07ac379c72754b149a39dca4586aeeb1d6285d98f60cd3c4454e1609cf944307;
    let caller = abi(MyContract, contract_id);

    let data = 1u64;
    let slice = raw_slice::from_parts::<u64>(__addr_of(&data), 1);
    let result = caller.test_function(slice);
    
    assert(result.len::<u8>() == slice.len::<u8>());
}
