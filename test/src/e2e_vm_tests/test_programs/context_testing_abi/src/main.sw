library context_testing_abi;
use std::contract_id::ContractId;

pub struct ParamsContractBalance {
    asset_id: b256,
    contract_id: ContractId
}

abi ContextTesting {
  fn get_id(gas: u64, coins: u64, asset_id: b256, input: ()) -> b256;
  fn get_this_balance(gas: u64, coins: u64, asset_id: b256, asset_id: b256) -> u64;
  fn get_balance_of_contract(gas: u64, coins: u64, asset_id: b256, params: ParamsContractBalance) -> u64;
  fn get_amount(gas: u64, coins: u64, asset_id: b256, input: ()) -> u64;
  fn get_asset_id(gas: u64, coins: u64, asset_id: b256, input: ()) -> b256;
  fn get_gas(gas: u64, coins: u64, asset_id: b256, input: ()) -> u64;
  fn get_global_gas(gas: u64, coins: u64, asset_id: b256, input: ()) -> u64;
}

