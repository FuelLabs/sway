use fuel_core::service::Config;
use fuel_tx::Salt;
use fuels_abigen_macro::abigen;
use fuels_contract::contract::Contract;
use fuels_signers::provider::Provider;

abigen!(
    TestFuelCoinContract,
    "test_projects/token_ops/out/debug/token_ops-abi.json"
);

#[tokio::test]
async fn mint() {
    let salt = Salt::from([0u8; 32]);
    let compiled = Contract::compile_sway_contract("test_projects/token_ops", salt).unwrap();
    let client = Provider::launch(Config::local_node()).await.unwrap();
    let id = Contract::deploy(&compiled, &client).await.unwrap();
    let instance = TestFuelCoinContract::new(id.to_string(), client);

    let c = testfuelcoincontract_mod::ContractId { value: id.into() };

    let balance_check_1 = ParamsGetBalance {
        target: id.into(),
        asset_id: c.clone(),
    };

    let mut balance_result = instance.get_balance(balance_check_1).call().await.unwrap();

    assert_eq!(balance_result.value, 0);

    instance.mint_coins(11).call().await.unwrap();

    let balance_check_2 = ParamsGetBalance {
        target: id.into(),
        asset_id: c.clone(),
    };

    balance_result = instance.get_balance(balance_check_2).call().await.unwrap();

    assert_eq!(balance_result.value, 11);
}

#[tokio::test]
async fn burn() {
    let salt = Salt::from([0u8; 32]);
    let compiled = Contract::compile_sway_contract("test_projects/token_ops", salt).unwrap();
    let client = Provider::launch(Config::local_node()).await.unwrap();
    let id = Contract::deploy(&compiled, &client).await.unwrap();
    let instance = TestFuelCoinContract::new(id.to_string(), client);

    let c = testfuelcoincontract_mod::ContractId { value: id.into() };

    let balance_check_1 = ParamsGetBalance {
        target: id.into(),
        asset_id: c.clone(),
    };

    let mut balance_result = instance.get_balance(balance_check_1).call().await.unwrap();

    assert_eq!(balance_result.value, 0);

    instance.mint_coins(11).call().await.unwrap();
    instance.burn_coins(7).call().await.unwrap();

    let balance_check_2 = ParamsGetBalance {
        target: id.into(),
        asset_id: c.clone(),
    };

    balance_result = instance.get_balance(balance_check_2).call().await.unwrap();

    assert_eq!(balance_result.value, 4);
}
