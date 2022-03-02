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
    let fuelcoin_id = ~ContractId::from(0xe92f8eda3411f75d4cb1f6cb8c9b0825421456de6b32e2bc8edcc992ecd2c0e4);
    let balance_test_id = ~ContractId::from(0x7e4f67697c313eea120d7fa62b49526f6451ea489a539bf0ebfd43cc5b5d9213);

    // todo: use correct type ContractId
    let fuel_coin = abi(TestFuelCoin, 0xe92f8eda3411f75d4cb1f6cb8c9b0825421456de6b32e2bc8edcc992ecd2c0e4);

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
