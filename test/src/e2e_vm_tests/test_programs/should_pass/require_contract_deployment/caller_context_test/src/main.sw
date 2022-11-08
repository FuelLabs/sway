script;
use std::{assert::assert, constants::BASE_ASSET_ID, contract_id::ContractId};
use context_testing_abi::*;
use core::num::*;

fn main() -> bool {
    let zero = b256::min();
    let gas: u64 = u64::max();
    let amount: u64 = 11;
    let other_contract_id = ContractId::from(0x78dc5cda1446f4ddbd1d2e1e688a3be0566caf5d2bde6525dc5b0da90660b9e2);
    let base_asset_id = BASE_ASSET_ID;

    let test_contract = abi(ContextTesting, other_contract_id.into());

    // test Context::contract_id():
    let returned_contract_id = test_contract.get_id {
        gas: gas, coins: 0, asset_id: BASE_ASSET_ID.into()
    }
    ();
    assert(returned_contract_id.into() == other_contract_id.into());

    // @todo set up a test contract to mint some tokens for testing balances.
    // test Context::this_balance():
    let returned_this_balance = test_contract.get_this_balance {
        gas: gas, coins: 0, asset_id: BASE_ASSET_ID.into()
    }
    (base_asset_id);
    assert(returned_this_balance == 0);

    // test Context::balance_of_contract():
    let returned_contract_balance = test_contract.get_balance_of_contract {
        gas: gas, coins: 0, asset_id: BASE_ASSET_ID.into()
    }
    (base_asset_id, other_contract_id);
    assert(returned_contract_balance == 0);

    // The checks below don't work (AssertIdNotFound). The test should be
    // updated to forward coins that are actually available.

    // test Context::msg_value():
    /*let returned_amount = test_contract.get_amount {
        gas: gas, coins: amount, asset_id: BASE_ASSET_ID
    }
    ();
    assert(returned_amount == amount);

    // test Context::msg_asset_id():
    let returned_asset_id = test_contract.get_asset_id {
        gas: gas, coins: amount, asset_id: BASE_ASSET_ID
    }
    ();
    assert(returned_asset_id.into() == BASE_ASSET_ID);

    // test Context::msg_gas():
    // @todo expect the correct gas here... this should fail using `1000`
    let gas = test_contract.get_gas {
        gas: gas, coins: 0, asset_id: BASE_ASSET_ID
    }
    ();
    assert(gas == 1000);

    // test Context::global_gas():
    // @todo expect the correct gas here... this should fail using `1000`
    let global_gas = test_contract.get_global_gas {
        gas: gas, coins: 0, asset_id: BASE_ASSET_ID
    }
    ();
    assert(global_gas == 1000);*/

    true
}
