script;

use std::context::balance_of;
use std::asset::*;
use std::contract_id::*;
use std::constants::DEFAULT_SUB_ID;
use test_fuel_coin_abi::*;

#[cfg(experimental_new_encoding = false)]
const FUEL_COIN_CONTRACT_ID = 0x9e84e7e3006c94d5e23545829dbc926f50f90c6809c6e599ecddbf9a119e8104;
#[cfg(experimental_new_encoding = true)]
const FUEL_COIN_CONTRACT_ID = 0xecd6eeff9cfd449b038e8b9d5025b7f58eb7959af26b1d84636aab641ecf8ebf;

#[cfg(experimental_new_encoding = false)]
const BALANCE_CONTRACT_ID = 0x3b8cb681056f61a41e138b8884d7e3bb9332fbd7a8e38e3e0b0ada766cabfa4e;
#[cfg(experimental_new_encoding = true)]
const BALANCE_CONTRACT_ID = 0xf14fd5b5d028dd6c0db49803f886deb2b1dc87de092857795d77c8a04a0926ad;

fn main() -> bool {
    let default_gas = 1_000_000_000_000;

    // the deployed fuel_coin Contract_Id:
    let fuelcoin_id = ContractId::from(FUEL_COIN_CONTRACT_ID);
    let fuelcoin_asset_id = AssetId::new(fuelcoin_id, DEFAULT_SUB_ID);

    // contract ID for sway/test/src/e2e_vm_tests/test_programs/should_pass/test_contracts/balance_test_contract/
    let balance_test_id = ContractId::from(BALANCE_CONTRACT_ID);

    // todo: use correct type ContractId
    let fuel_coin = abi(TestFuelCoin, fuelcoin_id.into());

    // Get the initial balances which can be non-zero
    // since we can't be sure if the contracts are fresh
    let fuelcoin_initial_balance = balance_of(fuelcoin_id, fuelcoin_asset_id);
    let balance_test_initial_balance = balance_of(fuelcoin_id, fuelcoin_asset_id);

    fuel_coin.mint {
        gas: default_gas
    }
    (11);

    // check that the mint was successful
    let fuelcoin_balance = balance_of(fuelcoin_id, fuelcoin_asset_id) - fuelcoin_initial_balance;
    assert(fuelcoin_balance == 11);

    fuel_coin.burn {
        gas: default_gas
    }
    (7);

    // check that the burn was successful
    let fuelcoin_balance = balance_of(fuelcoin_id, fuelcoin_asset_id) - fuelcoin_initial_balance;
    assert(fuelcoin_balance == 4);

    // force transfer coins
    fuel_coin.force_transfer {
        gas: default_gas
    }
    (3, fuelcoin_asset_id, balance_test_id);

    // check that the transfer was successful
    let fuelcoin_balance = balance_of(fuelcoin_id, fuelcoin_asset_id) - fuelcoin_initial_balance;
    let balance_test_contract_balance = balance_of(balance_test_id, fuelcoin_asset_id) - balance_test_initial_balance;
    assert(fuelcoin_balance == 1);
    assert(balance_test_contract_balance == 3);

    true
}