library;

abi ContextTesting {
    fn get_id() -> ContractId;
    fn get_this_balance(asset_id: b256) -> u64;
    fn get_balance_of_contract(asset_id: b256, contract_id: ContractId) -> u64;
    fn get_amount() -> u64;
    fn get_asset_id() -> b256;
    fn get_gas() -> u64;
    fn get_global_gas() -> u64;
}
