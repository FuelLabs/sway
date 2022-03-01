script;
use std::{chain::assert, constants::{ETH_ID, ZERO}, contract_id::ContractId};
use context_testing_abi::*;

fn main() -> bool {
    let gas: u64 = 1000;
    let amount: u64 = 11;
    let other_contract_id = ~ContractId::from(0x27829e78404b18c037b15bfba5110c613a83ea22c718c8b51596e17c9cb1cd6f);
    let deployed_contract_id = 0x2890a0a3fb38e88d4ef887d78db0cd4483583a00506c30b572dff1aa73305b3e;

    let test_contract = abi(ContextTesting, deployed_contract_id);

    // test Context::contract_id():
    let returned_contract_id = test_contract.get_id {
        gas: gas, coins: 0, asset_id: ETH_ID
    }
    ();
    assert(returned_contract_id == deployed_contract_id);

    // @todo set up a test contract to mint some tokens for testing balances.
    // test Context::this_balance():
    let returned_this_balance = test_contract.get_this_balance {
        gas: gas, coins: 0, asset_id: ETH_ID
    }
    (ETH_ID);
    assert(returned_this_balance == 0);

    // test Context::balance_of_contract():
    let returned_contract_balance = test_contract.get_balance_of_contract {
        gas: gas, coins: 0, asset_id: ETH_ID
    }
    (ETH_ID, other_contract_id);
    assert(returned_contract_balance == 0);

    // test Context::msg_value():
    let returned_amount = test_contract.get_amount {
        gas: gas, coins: amount, asset_id: ETH_ID
    }
    ();
    assert(returned_amount == amount);

    // test Context::msg_asset_id():
    let returned_asset_id = test_contract.get_asset_id {
        gas: gas, coins: amount, asset_id: ETH_ID
    }
    ();
    assert(returned_asset_id == ETH_ID);

    // test Context::msg_gas():
    // @todo expect the correct gas here... this should fail using `1000`
    let gas = test_contract.get_gas {
        gas: gas, coins: amount, asset_id: ETH_ID
    }
    ();
    assert(gas == 1000);

    // test Context::global_gas():
    // @todo expect the correct gas here... this should fail using `1000`
    let global_gas = test_contract.get_global_gas {
        gas: gas, coins: amount, asset_id: ETH_ID
    }
    ();
    assert(global_gas == 1000);

    true
}
