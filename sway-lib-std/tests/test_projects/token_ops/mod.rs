use fuel_tx::{ContractId, Salt};
use fuels_abigen_macro::abigen;
use fuels_contract::{contract::Contract, parameters::TxParameters};
use fuels_signers::util::test_helpers;

abigen!(
    TestFuelCoinContract,
    "test_projects/token_ops/out/debug/token_ops-abi.json"
);

#[tokio::test]
async fn can_mint() {
    let (fuelcoin_instance, fuelcoin_id) = get_fuelcoin_instance().await;

    let target = testfuelcoincontract_mod::ContractId { value: fuelcoin_id.into() };
    let asset_id = target;

    let mut balance_result = fuelcoin_instance
        .get_balance(target.clone(), asset_id.clone())
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 0);

    fuelcoin_instance.mint_coins(11).call().await.unwrap();

    balance_result = fuelcoin_instance.get_balance(target, asset_id).call().await.unwrap();
    assert_eq!(balance_result.value, 11);
}

#[tokio::test]
async fn can_burn() {
    let (fuelcoin_instance, fuelcoin_id) = get_fuelcoin_instance().await;

    let target = testfuelcoincontract_mod::ContractId { value: fuelcoin_id.into() };
    // let asset_id = testfuelcoincontract_mod::ContractId { value: fuelcoin_id.into() };
    let asset_id = target;

    let mut balance_result = fuelcoin_instance
        .get_balance(target.clone(), asset_id.clone())
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 0);

    fuelcoin_instance.mint_coins(11).call().await.unwrap();
    fuelcoin_instance.burn_coins(7).call().await.unwrap();

    balance_result = fuelcoin_instance.get_balance(target, asset_id).call().await.unwrap();
    assert_eq!(balance_result.value, 4);
}

#[tokio::test]
async fn can_force_transfer() {
    let (fuelcoin_instance, fuelcoin_id) = get_fuelcoin_instance().await;
    let balance_id = get_balance_contract_id().await;

    let asset_id = testfuelcoincontract_mod::ContractId {
        value: fuelcoin_id.into(),
    };

    let target = testfuelcoincontract_mod::ContractId {
        value: balance_id.into(),
    };

    let mut balance_result = fuel_coin_instance.get_balance(asset_id.clone(), asset_id.clone()).call().await.unwrap();
    assert_eq!(balance_result.value, 0);

    fuel_coin_instance
        .mint_coins(100)
        .call()
        .await
        .unwrap();

    balance_result = fuel_coin_instance.get_balance(asset_id.clone(), asset_id.clone()).call().await.unwrap();
    assert_eq!(balance_result.value, 100);

    // confirm initial balance on balance contract (recipient)
    balance_result = fuel_coin_instance.get_balance(asset_id.clone(), target.clone()).set_contracts(&[balance_contract_id]).call().await.unwrap();
    assert_eq!(balance_result.value, 0);

    let coins = 42u64;

    fuel_coin_instance
        .force_transfer_coins(coins, asset_id.clone(), target.clone())
        .set_contracts(&[fuelcoin_id, balance_contract_id])
        .call()
        .await
        .unwrap();

    // confirm remaining balance on fuelcoin contract
    balance_result = fuel_coin_instance.get_balance(asset_id.clone(), asset_id.clone()).call().await.unwrap();
    assert_eq!(balance_result.value, 58);

    // confirm new balance on balance contract (recipient)
    balance_result = fuel_coin_instance.get_balance(asset_id.clone(), target.clone()).set_contracts(&[balance_contract_id]).call().await.unwrap();
    assert_eq!(balance_result.value, 42);
}

async fn get_fuelcoin_instance() -> (TestFuelCoinContract, ContractId) {
    let salt = Salt::from([0u8; 32]);
    let compiled =
        Contract::load_sway_contract("test_projects/token_ops/out/debug/token_ops.bin", salt)
            .unwrap();
    let (provider, wallet) = test_helpers::setup_test_provider_and_wallet().await;
    let fuelcoin_id = Contract::deploy(&compiled, &provider, &wallet, TxParameters::default())
        .await
        .unwrap();

    let fuelcoin_instance = TestRegistersContract::new(id.to_string(), provider, wallet);

    (fuelcoin_instance, fuelcoin_id)
}

async fn get_balance_contract_id() -> ContractId {
    let salt = Salt::from([0u8; 32]);
    let compiled =
        Contract::load_sway_contract("test_artifacts/balance_contract/out/debug/balance_contract.bin", salt)
            .unwrap();
    let (provider, wallet) = test_helpers::setup_test_provider_and_wallet().await;
    let balance_id = Contract::deploy(&compiled, &provider, &wallet, TxParameters::default())
        .await
        .unwrap();

    balance_id
}
