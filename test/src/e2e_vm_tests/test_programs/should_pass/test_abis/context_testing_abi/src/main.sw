library;

abi ContextTesting {
    fn get_id() -> ContractId;
    fn get_this_balance(asset_id: AssetId) -> u64;
    fn get_balance_of_contract(asset_id: AssetId, contract_id: ContractId) -> u64;
    fn get_amount() -> u64;
    fn get_asset_id() -> AssetId;
    fn get_gas() -> u64;
    fn get_global_gas() -> u64;
}
