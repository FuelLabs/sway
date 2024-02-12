script;

use std::constants::BASE_ASSET_ID;
use context_testing_abi::*;

fn main() -> bool {
    let gas: u64 = u64::max();
    let amount: u64 = 11;
    let other_contract_id = ContractId::from(0x6cfe6fe68a7199fc628df977cc100912a17496b9736b1d98c069ea1fff05502f);
    let other_contract_id_b256: b256 = other_contract_id.into();
    let base_asset_id = BASE_ASSET_ID;

    let test_contract = abi(ContextTesting, other_contract_id_b256);

    // test Context::contract_id():
    let returned_contract_id = test_contract.get_id {
        gas: gas,
        coins: 0,
        asset_id: BASE_ASSET_ID.value,
    }();
    let returned_contract_id_b256: b256 = returned_contract_id.into();
    assert(returned_contract_id_b256 == other_contract_id_b256);

    // @todo set up a test contract to mint some assets for testing balances.
    // test Context::this_balance():
    let returned_this_balance = test_contract.get_this_balance {
        gas: gas,
        coins: 0,
        asset_id: BASE_ASSET_ID.value,
    }(base_asset_id);
    assert(returned_this_balance == 0);

    // test Context::balance_of_contract():
    let returned_contract_balance = test_contract.get_balance_of_contract {
        gas: gas,
        coins: 0,
        asset_id: BASE_ASSET_ID.value,
    }(base_asset_id, other_contract_id);
    assert(returned_contract_balance == 0);

    // TODO: The checks below don't work (AssertIdNotFound). The test should be
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
