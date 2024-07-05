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
    let contract_id = 0xfef18ef24b6cbfd66238fecc3c2704976fdf3177442712a3402b2ab666f12039; // AUTO-CONTRACT-ID .
    let caller = abi(MyContract, contract_id); 
    let result = caller.test_function("a");
    assert(result == "a")
}
