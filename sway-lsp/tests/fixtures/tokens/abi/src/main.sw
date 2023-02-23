contract;

struct Empty{}

abi MyContract {
    fn test_function() -> Empty;
}

impl MyContract for Contract {
    fn test_function() -> Empty {
        Empty{}
    }
}

fn caller(address: ContractId) -> ContractCaller<_> {
    abi(MyContract, address.value)
}
