contract;

abi MyContract {
    fn test_function(a: str) -> str;
}

impl MyContract for Contract {
    fn test_function(a: str) -> str {
        a
    }
}

#[test]
fn test_success() {
    let contract_id = 0xdf476984192908f69312709c24090819fd64b3d2efa974bafd616c1811cd5572;
    let caller = abi(MyContract, contract_id);
    let result = caller.test_function("a");
    assert(result == "a")
}
