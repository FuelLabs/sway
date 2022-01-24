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
    let fuelcoin_id = ~ContractId::from(0x49050d88db785fdf3b3fd35df44fbe644b9b5273ae97d9c7c19cc3e4aa361eaf);
    // todo: use correct type ContractId
    let fuel_coin = abi(TestFuelCoin, 0x49050d88db785fdf3b3fd35df44fbe644b9b5273ae97d9c7c19cc3e4aa361eaf);

    let mut fuelcoin_balance = balance_of_contract(fuelcoin_id.value, fuelcoin_id);
    assert(fuelcoin_balance == 0);

    fuel_coin.mint(default.gas, default.coins, default.id.value, 11);

    // check that the mint was successful
    fuelcoin_balance = balance_of_contract(fuelcoin_id.value, fuelcoin_id);
    assert(fuelcoin_balance == 11);

    fuel_coin.burn(default.gas, default.coins, default.id.value, 7);

    // check that the burn was successful
    fuelcoin_balance = balance_of_contract(fuelcoin_id.value, fuelcoin_id);
    assert(fuelcoin_balance == 4);

    true
}
