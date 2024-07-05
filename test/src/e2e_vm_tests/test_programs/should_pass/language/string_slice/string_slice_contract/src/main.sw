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
    let contract_id = fuel]; // AUTO-CONTRACT-ID .
    let caller = abi(MyContract, contract_id); 
    let result = caller.test_function("a");
    assert(result == "a")
}
