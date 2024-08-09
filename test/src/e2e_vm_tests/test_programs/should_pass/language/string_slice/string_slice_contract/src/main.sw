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
    let contract_id = 0x1f66c7619144fcc4f2b7aebfbf4b84fa7ff7881f8d1b3cbea650135c585683c6; // AUTO-CONTRACT-ID .
    let caller = abi(MyContract, contract_id); 
    let result = caller.test_function("a");
    assert(result == "a")
}
