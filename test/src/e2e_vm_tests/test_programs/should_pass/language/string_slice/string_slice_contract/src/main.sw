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
    let contract_id = 0x403f0f71c484bf32babd07717ea3c3ff0710f47a9b9aef8023292fa2b740f6c6;
    let caller = abi(MyContract, contract_id);
    let result = caller.test_function("a");
    assert(result == "a")
}
