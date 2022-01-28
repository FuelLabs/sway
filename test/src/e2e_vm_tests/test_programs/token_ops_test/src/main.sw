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
    let fuelcoin_id = ~ContractId::from(0x45a3c8a90756c423e35655d1d0f13c4c32206c39457b7825c823cef7976676e4);
    let balance_test_id = ~ContractId::from(0x27eb552a9458aec1db874930ae86fe91df49b4e0c221e08f7ffcf3fadadee0a3);
    let test_recipient = ~Address::from(0x3333333333333333333333333333333333333333333333333333333333333333);

    // todo: use correct type ContractId
    let fuel_coin = abi(TestFuelCoin, 0x45a3c8a90756c423e35655d1d0f13c4c32206c39457b7825c823cef7976676e4);

    let mut fuelcoin_balance = balance_of_contract(fuelcoin_id.value, fuelcoin_id);
    assert(fuelcoin_balance == 0);

    fuel_coin.mint_coins(default.gas, default.coins, default.id.value, 11);

    // check that the mint was successful
    fuelcoin_balance = balance_of_contract(fuelcoin_id.value, fuelcoin_id);
    assert(fuelcoin_balance == 11);

    fuel_coin.burn_coins(default.gas, default.coins, default.id.value, 7);

    // check that the burn was successful
    fuelcoin_balance = balance_of_contract(fuelcoin_id.value, fuelcoin_id);
    assert(fuelcoin_balance == 4);

    let force_transfer_args = ParamsForceTransfer {
        coins: 3,
        asset_id: fuelcoin_id,
        c_id: balance_test_id,
    };

    // force transfer coins
    fuel_coin.force_transfer_coins(default.gas, default.coins, default.id.value, force_transfer_args);

    // check that the transfer was successful
    fuelcoin_balance = balance_of_contract(fuelcoin_id.value, fuelcoin_id);
    let balance_test_contract_balance = balance_of_contract(fuelcoin_id.value, balance_test_id);
    assert(fuelcoin_balance == 1);
    assert(balance_test_contract_balance == 3);

    let transfer_to_output_args = ParamsTransferToOutput {
        coins: 1,
        asset_id: fuelcoin_id,
        recipient: test_recipient,
    };

    // transfer coins to output
    fuel_coin.transfer_coins_to_output(default.gas, default.coins, default.id.value, transfer_to_output_args);

    // check that the transfer was a success
    // TODO: additional testing to check the recipient's balance ?
    fuelcoin_balance = balance_of_contract(fuelcoin_id.value, fuelcoin_id);
    assert(fuelcoin_balance == 0);

    true
}
