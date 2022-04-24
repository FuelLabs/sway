library context_testing_abi;
use std::contract_id::ContractId;

abi ContextTesting {
    fn get_this_balance(asset: ContractId) -> u64;
    fn get_balance_of_contract(asset: ContractId, contract: ContractId) -> u64;
    fn get_amount() -> u64;
    fn get_asset_id() -> ContractId;
    fn get_gas() -> u64;
    fn get_global_gas() -> u64;
    fn receive_coins();
}
