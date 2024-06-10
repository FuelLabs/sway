script;

use std::context::balance_of;
use std::asset::*;
use std::contract_id::*;
use std::constants::DEFAULT_SUB_ID;
use test_fuel_coin_abi::*;

#[cfg(experimental_new_encoding = false)]
const FUEL_COIN_CONTRACT_ID = 0x4c7b43ef5a097d7cfb87600a4234e33311eeeeb8081e5ea7bb6d9a1f8555c9c4;
#[cfg(experimental_new_encoding = true)]
const FUEL_COIN_CONTRACT_ID = 0xbbe32b92c842a1d719bcd5f1610fb6193390ace397edd34c6722236a5d6b26c9;

#[cfg(experimental_new_encoding = false)]
const BALANCE_CONTRACT_ID = 0x3120fdd1b99c0c611308aff43a99746cc2c661c69c22aa56331d5f3ce5534ee9;
#[cfg(experimental_new_encoding = true)]
const BALANCE_CONTRACT_ID = 0xef1627d701e2a5f0b0fc35154b5f61da1aafbf597b41d639251304b0d03dfb3c;

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
