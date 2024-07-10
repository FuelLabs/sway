contract;

struct Empty {}

/// Docs for MyContract
abi MyContract {
    fn test_function() -> Empty;
}

impl MyContract for Contract {
    fn test_function() -> Empty {
        Empty {}
    }
}

fn caller(address: b256) -> ContractCaller<_> {
    abi(MyContract, address)
}
