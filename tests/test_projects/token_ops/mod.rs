use fuel_core::service::{Config, FuelService};
use fuel_tx::Salt;
use fuels_abigen_macro::abigen;
use fuels_contract::{parameters::TxParameters, contract::Contract};
use fuels_signers::util::test_helpers;


abigen!(
    TestFuelCoinContract,
    "test_projects/token_ops/out/debug/token_ops-abi.json"
);

#[tokio::test]
async fn mint() {
    let salt = Salt::from([0u8; 32]);
    let compiled =
        Contract::load_sway_contract("test_projects/token_ops/out/debug/token_ops.bin", salt)
            .unwrap();

    let server = FuelService::new_node(Config::local_node()).await.unwrap();
    let (provider, wallet) = test_helpers::setup_test_provider_and_wallet().await;
    let id = Contract::deploy(&compiled, &provider, &wallet, TxParameters::default()).await.unwrap();
    
    let instance = TestFuelCoinContract::new(id.to_string(), provider, wallet);

    let target = testfuelcoincontract_mod::ContractId { value: id.into() };
    let asset_id = testfuelcoincontract_mod::ContractId { value: id.into() };

    let mut balance_result = instance
        .get_balance(target.clone(), asset_id.clone())
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 0);

    instance.mint_coins(11).call().await.unwrap();

    balance_result = instance.get_balance(target, asset_id).call().await.unwrap();
    assert_eq!(balance_result.value, 11);
}

#[tokio::test]
async fn burn() {
    let salt = Salt::from([0u8; 32]);
    let compiled =
        Contract::load_sway_contract("test_projects/token_ops/out/debug/token_ops.bin", salt)
            .unwrap();
            
    let server = FuelService::new_node(Config::local_node()).await.unwrap();
    let (provider, wallet) = test_helpers::setup_test_provider_and_wallet().await;
    let id = Contract::deploy(&compiled, &provider, &wallet, TxParameters::default()).await.unwrap();
    
    let instance = TestFuelCoinContract::new(id.to_string(), provider, wallet);

    let target = testfuelcoincontract_mod::ContractId { value: id.into() };
    let asset_id = testfuelcoincontract_mod::ContractId { value: id.into() };

    let mut balance_result = instance
        .get_balance(target.clone(), asset_id.clone())
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 0);

    instance.mint_coins(11).call().await.unwrap();
    instance.burn_coins(7).call().await.unwrap();

    balance_result = instance.get_balance(target, asset_id).call().await.unwrap();
    assert_eq!(balance_result.value, 4);
}
