library context_testing_abi;

abi ContextTesting {
    #[payable]
    fn get_this_balance(asset: ContractId) -> u64;
    #[payable]
    fn get_balance_of_contract(asset: ContractId, r#contract: ContractId) -> u64;
    #[payable]
    fn get_amount() -> u64;
    #[payable]
    fn get_asset_id() -> ContractId;
    #[payable]
    fn get_gas() -> u64;
    #[payable]
    fn get_global_gas() -> u64;
    #[payable]
    fn receive_coins();
}
