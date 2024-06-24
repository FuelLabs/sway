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
    let contract_id = 0xe1669acd3f6d70b9ede9953ae1694225c8005416f4d8e01017070d4d3f9c4254; // AUTO-CONTRACT-ID .
    let caller = abi(MyContract, contract_id); 
    let result = caller.test_function("a");
    assert(result == "a")
}
