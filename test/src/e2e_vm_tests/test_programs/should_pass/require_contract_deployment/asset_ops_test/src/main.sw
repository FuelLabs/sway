script;

use std::context::balance_of;
use std::asset::*;
use std::contract_id::*;
use std::constants::DEFAULT_SUB_ID;
use test_fuel_coin_abi::*;

#[cfg(experimental_new_encoding = false)]
const FUEL_COIN_CONTRACT_ID = 0x542c6e67e5e8768a2c119a80ddcbd1f8d01110ced16fda37e4aa77ebb6d6cdb9;
#[cfg(experimental_new_encoding = true)]
<<<<<<< HEAD
const FUEL_COIN_CONTRACT_ID = 0x8d3f7e1c4ae4c23e267f3d4b8f389a8d9e3753ab7adf455224062e49f69886f4;
=======
const FUEL_COIN_CONTRACT_ID = 0xe690b0d8891fd6fcf66c9801b4cd770b46514dd91a06c08f975ebe196013d2d5;
>>>>>>> 5a1a9d79c (updating contract ids)

#[cfg(experimental_new_encoding = false)]
const BALANCE_CONTRACT_ID = 0xe50966cd6b1da8fe006e3e876e08f3df6948ce426e1a7cfe49fba411b0a11f89;
#[cfg(experimental_new_encoding = true)]
<<<<<<< HEAD
const BALANCE_CONTRACT_ID = 0xb0b0589ced70b31fb34cbb7fbb1b0e4046cc61c2ffe79cdb06a617bf24d9458c;
=======
const BALANCE_CONTRACT_ID = 0xda15c092dd8c53c1f99a5b67bede3e497c9e178be0e22047a2af25b8036f041b;
>>>>>>> 5a1a9d79c (updating contract ids)

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