script;

// use std::constants::ETH_ID;
use std::context::balance_of_contract;
use std::chain::assert;
use std::address::Address;
use std::contract_id::ContractId;
use std::token::*;
use test_fuel_coin_abi::*;

struct Opts {
    gas: u64,
    coins: u64,
    id: ContractId,
}

fn main() -> bool {
    let default = Opts {
        gas: 1_000_000_000_000,
        coins: 0,
        id: ~ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000),
    };

    // the deployed fuel_coin Contract_Id:
    let fuelcoin_id = ~ContractId::from(0xff95564b8f788b6a2a884813eadfff2dbfe008a881008e7b298ce14208a73864);

    // contract ID for sway/test/src/e2e_vm_tests/test_programs/balance_test_contract
    let balance_test_id = ~ContractId::from(0xa835193dabf3fe80c0cb62e2ecc424f5ac03bc7f5c896ecc4bd2fd06cc434322);

    // todo: use correct type ContractId
    let fuel_coin = abi(TestFuelCoin, 0xff95564b8f788b6a2a884813eadfff2dbfe008a881008e7b298ce14208a73864);

    let mut fuelcoin_balance = balance_of_contract(fuelcoin_id.value, fuelcoin_id);
    assert(fuelcoin_balance == 0);

    fuel_coin.mint {
        gas: default.gas, coins: default.coins, asset_id: default.id.value
    }
    (11);

    // check that the mint was successful
    fuelcoin_balance = balance_of_contract(fuelcoin_id.value, fuelcoin_id);
    assert(fuelcoin_balance == 11);

    fuel_coin.burn {
        gas: default.gas, coins: default.coins, asset_id: default.id.value
    }
    (7);

    // check that the burn was successful
    fuelcoin_balance = balance_of_contract(fuelcoin_id.value, fuelcoin_id);
    assert(fuelcoin_balance == 4);

    // force transfer coins
    fuel_coin.force_transfer {
        gas: default.gas, coins: default.coins, asset_id: default.id.value
    }
    (3, fuelcoin_id, balance_test_id);

    // check that the transfer was successful
    fuelcoin_balance = balance_of_contract(fuelcoin_id.value, fuelcoin_id);
    let balance_test_contract_balance = balance_of_contract(fuelcoin_id.value, balance_test_id);
    assert(fuelcoin_balance == 1);
    assert(balance_test_contract_balance == 3);

    true
}
