script;

use std::constants::ETH_ID;
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
        gas: 1000,
        coins: 0,
        id: ~ContractId::from(ETH_ID),
    };

    let test_recipient = ~Address::from(0x3333333333333333333333333333333333333333333333333333333333333333);
    // the already deployed balance_test contract
    let balance_id = ~ContractId::from(0xa72b68c70be7e137de429840d67bce3b1e9a545fa05f77ec090091539d4fbf3c);

    // the deployed fuel_coin contract
    let fuelcoin_id = ~ContractId::from(0x9c8a446c98b85592823934520a4865a5a93b8dbb0e825e98ef26a08a6e88a17b);
    // todo: use correct type ContractId
    let fuel_coin = abi(TestFuelCoin, 0x9c8a446c98b85592823934520a4865a5a93b8dbb0e825e98ef26a08a6e88a17b);

    let mut fuelcoin_balance = balance_of_contract(fuelcoin_id.value, balance_id);
    assert(fuelcoin_balance == 0);

    fuel_coin.mint(default.gas, default.coins, default.id.value, 11);

    // check that the mint was successful
    fuelcoin_balance = balance_of_contract(fuelcoin_id.value, fuelcoin_id);
    assert(fuelcoin_balance == 11);

    fuel_coin.burn(default.gas, default.coins, default.id.value, 7);

    // check that the burn was successful
    fuelcoin_balance = balance_of_contract(fuelcoin_id.value, fuelcoin_id);
    assert(fuelcoin_balance == 4);

    let force_transfer_args = ParamsForceTransfer {
        coins: 3,
        asset_id: fuelcoin_id,
        c_id: balance_id,
    };
    let mut balance2 = balance_of_contract(fuelcoin_id.value, balance_id);
    assert(balance2 == 0);

    fuel_coin.force_transfer(default.gas, default.coins, default.id.value, force_transfer_args);

    balance2 = balance_of_contract(fuelcoin_id.value, balance_id);
    fuelcoin_balance = balance_of_contract(fuelcoin_id.value, fuelcoin_id);
    assert(balance2 == 3);

    assert(fuelcoin_balance == 1);

    let transfer_to_output_args = ParamsTransferToOutput {
        coins: 1,
        asset_id: fuelcoin_id,
        recipient: test_recipient,
    };

    fuel_coin.transfer_to_output(default.gas, default.coins, default.id.value, transfer_to_output_args);
    fuelcoin_balance = balance_of_contract(fuelcoin_id.value, fuelcoin_id);
    assert(fuelcoin_balance == 0);

    true
}
