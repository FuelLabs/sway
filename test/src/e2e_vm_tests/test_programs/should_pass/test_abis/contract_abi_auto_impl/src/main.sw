contract;

impl Contract {
    fn impl_method() -> u64 { 42 }
}

#[test]
fn tests() {
    // let caller = abi(ContractAbiAutoImplAbi, CONTRACT_ID); // If tested from within project's directory
    let caller = abi(AnonymousAbi, CONTRACT_ID); // If tested from outside project's directory
    assert(caller.impl_method() == 42)
}
