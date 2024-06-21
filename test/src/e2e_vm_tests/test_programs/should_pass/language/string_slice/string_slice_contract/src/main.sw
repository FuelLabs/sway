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
    let contract_id = 0xd2cf22567d02b44ac9f2342b814d9e6502820fec8c37c9b4bc8e15a2821d329e;
    let caller = abi(MyContract, contract_id);
    let result = caller.test_function("a");
    assert(result == "a")
}
