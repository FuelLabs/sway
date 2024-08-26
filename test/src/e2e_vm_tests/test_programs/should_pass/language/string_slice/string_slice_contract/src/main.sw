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
    let contract_id = 0x573a7901ce5a722d0b80a4ad49296f8e6a23e4f6282555a561ed5118e5890ec2; // AUTO-CONTRACT-ID .
    let caller = abi(MyContract, contract_id); 
    let result = caller.test_function("a");
    assert(result == "a")
}
