script;

use std::context::balance_of;
use std::token::*;
use std::contract_id::*;
use std::constants::DEFAULT_SUB_ID;
use test_fuel_coin_abi::*;

struct Opts {
    gas: u64,
    coins: u64,
    id: ContractId,
}

fn main() -> bool {
    let default_gas = 1_000_000_000_000;

    // the deployed fuel_coin Contract_Id:
    let fuelcoin_id = ContractId::from(0x5d10689c7eecb405937a3f35fab7baf05a3f6189f9a2993ee70e21ccc1212460);
    let fuelcoin_asset_id = AssetId::new(fuelcoin_id, DEFAULT_SUB_ID);

    // contract ID for sway/test/src/e2e_vm_tests/test_programs/should_pass/test_contracts/balance_test_contract/
    let balance_test_id = ContractId::from(0x4a00baa517980432b9274a0e2f03c88735bdb483730816679c6eb37b4046d060);

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