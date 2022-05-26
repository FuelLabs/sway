library context_testing_abi;
use std::contract_id::ContractId;

abi ContextTesting {
    fn get_id() -> ContractId;
    fn get_this_balance(asset_id: ContractId) -> u64;
    fn get_balance_of_contract(asset_id: ContractId, contract_id: ContractId) -> u64;
    fn get_amount() -> u64;
    fn get_asset_id() -> ContractId;
    fn get_gas() -> u64;
    fn get_global_gas() -> u64;
}
