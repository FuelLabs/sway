script;

use std::assert::assert;
use std::address::Address;
use std::context::balance_of;
use std::contract_id::ContractId;
use std::token::*;
use test_fuel_coin_abi::*;

struct Opts {
    gas: u64,
    coins: u64,
    id: ContractId,
}

fn main() -> bool {
    let default_gas = 1_000_000_000_000;

    // the deployed fuel_coin Contract_Id:
    let fuelcoin_id = ~ContractId::from(0xb428d933827462e8a4ec8ae05c990bf9f2fde36a8b843acd015bbd98a39ef2f6);

    // contract ID for sway/test/src/e2e_vm_tests/test_programs/should_pass/test_contracts/balance_test_contract/
    let balance_test_id = ~ContractId::from(0x11dc3309952fa0f6d65abf1f57bc1b7fafca29459a8050d6eb44bce2241c2aa0);

    // todo: use correct type ContractId
    let fuel_coin = abi(TestFuelCoin, fuelcoin_id.into());

    let mut fuelcoin_balance = balance_of(fuelcoin_id, fuelcoin_id);
    assert(fuelcoin_balance == 0);

    fuel_coin.mint {
        gas: default_gas
    }
    (11);

    // check that the mint was successful
    fuelcoin_balance = balance_of(fuelcoin_id, fuelcoin_id);
    assert(fuelcoin_balance == 11);

    fuel_coin.burn {
        gas: default_gas
    }
    (7);

    // check that the burn was successful
    fuelcoin_balance = balance_of(fuelcoin_id, fuelcoin_id);
    assert(fuelcoin_balance == 4);

    // force transfer coins
    fuel_coin.force_transfer {
        gas: default_gas
    }
    (3, fuelcoin_id, balance_test_id);

    // check that the transfer was successful
    fuelcoin_balance = balance_of(fuelcoin_id, fuelcoin_id);
    let balance_test_contract_balance = balance_of(fuelcoin_id, balance_test_id);
    assert(fuelcoin_balance == 1);
    assert(balance_test_contract_balance == 3);

    true
}
