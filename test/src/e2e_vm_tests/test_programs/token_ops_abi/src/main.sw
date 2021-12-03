library token_ops_abi;

abi TokenOps {
    fn mint(gas: u64, coins: u64, color: b256, input: ());
    fn burn(gas: u64, coins: u64, color: b256, input: ());
    fn transfer_to_output(gas: u64, coins: u64, color: b256, input: ());
}
