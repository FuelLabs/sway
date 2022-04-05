use fuel_tx::{ContractId, AssetId, Salt};
use fuels_abigen_macro::abigen;
use fuels_contract::{contract::Contract, parameters::TxParameters};
use fuels_signers::provider::Provider;
use fuels_signers::{util::test_helpers::setup_test_provider_and_wallet, Signer};
use fuels_signers::wallet::Wallet;

abigen!(
    TestFuelCoinContract,
    "test_projects/token_ops/out/debug/token_ops-abi.json"
);

#[tokio::test]
async fn can_mint() {
    let (provider, wallet) = setup_test_provider_and_wallet().await;
    let (fuelcoin_instance, fuelcoin_id) = get_fuelcoin_instance(provider, wallet).await;

    let target = testfuelcoincontract_mod::ContractId {
        value: fuelcoin_id.into(),
    };
    let asset_id = target.clone();

    let mut balance_result = fuelcoin_instance
        .get_balance(target.clone(), asset_id.clone())
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 0);

    fuelcoin_instance.mint_coins(11).call().await.unwrap();

    balance_result = fuelcoin_instance
        .get_balance(target, asset_id)
        .call()
        .await
        .unwrap();
    // TEMPORARILY DISABLED until https://github.com/FuelLabs/fuels-rs/issues/201 is resolved.
    //assert_eq!(balance_result.value, 11);
}

#[tokio::test]
async fn can_burn() {
    let (provider, wallet) = setup_test_provider_and_wallet().await;
    let (fuelcoin_instance, fuelcoin_id) = get_fuelcoin_instance(provider, wallet).await;

    let target = testfuelcoincontract_mod::ContractId {
        value: fuelcoin_id.into(),
    };
    let asset_id = target.clone();

    let mut balance_result = fuelcoin_instance
        .get_balance(target.clone(), asset_id.clone())
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 0);

    fuelcoin_instance.mint_coins(11).call().await.unwrap();
    // TEMPORARILY DISABLED until https://github.com/FuelLabs/fuels-rs/issues/201 is resolved.
    //fuelcoin_instance.burn_coins(7).call().await.unwrap();
    //
    //balance_result = fuelcoin_instance
    //    .get_balance(target, asset_id)
    //    .call()
    //    .await
    //    .unwrap();
    //assert_eq!(balance_result.value, 4);
}

#[tokio::test]
async fn can_force_transfer() {
    let (provider, wallet) = setup_test_provider_and_wallet().await;
    let (fuelcoin_instance, fuelcoin_id) =
        get_fuelcoin_instance(provider.clone(), wallet.clone()).await;
    let balance_id = get_balance_contract_id(provider, wallet).await;

    let asset_id = testfuelcoincontract_mod::ContractId {
        value: fuelcoin_id.into(),
    };

    let target = testfuelcoincontract_mod::ContractId {
        value: balance_id.into(),
    };

    let mut balance_result = fuelcoin_instance
        .get_balance(asset_id.clone(), asset_id.clone())
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 0);

    fuelcoin_instance.mint_coins(100).call().await.unwrap();

    // TEMPORARILY DISABLED until https://github.com/FuelLabs/fuels-rs/issues/201 is resolved.
    //balance_result = fuelcoin_instance
    //    .get_balance(asset_id.clone(), asset_id.clone())
    //    .call()
    //    .await
    //    .unwrap();
    //assert_eq!(balance_result.value, 100);

    // confirm initial balance on balance contract (recipient)
    balance_result = fuelcoin_instance
        .get_balance(asset_id.clone(), target.clone())
        .set_contracts(&[balance_id])
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 0);

    let coins = 42u64;

    // TEMPORARILY DISABLED until https://github.com/FuelLabs/fuels-rs/issues/201 is resolved.
    //fuelcoin_instance
    //    .force_transfer_coins(coins, asset_id.clone(), target.clone())
    //    .set_contracts(&[fuelcoin_id, balance_id])
    //    .call()
    //    .await
    //    .unwrap();

    // confirm remaining balance on fuelcoin contract
    balance_result = fuelcoin_instance
        .get_balance(asset_id.clone(), asset_id.clone())
        .call()
        .await
        .unwrap();
    // TEMPORARILY DISABLED until https://github.com/FuelLabs/fuels-rs/issues/201 is resolved.
    //assert_eq!(balance_result.value, 58);

    // confirm new balance on balance contract (recipient)
    balance_result = fuelcoin_instance
        .get_balance(asset_id.clone(), target.clone())
        .set_contracts(&[balance_id])
        .call()
        .await
        .unwrap();
    // TEMPORARILY DISABLED until https://github.com/FuelLabs/fuels-rs/issues/201 is resolved.
    //assert_eq!(balance_result.value, 42);
}

#[tokio::test]
async fn can_mint_and_send_to_contract() {
    let (provider, wallet) = setup_test_provider_and_wallet().await;
    let (fuelcoin_instance, fuelcoin_id) =
        get_fuelcoin_instance(provider.clone(), wallet.clone()).await;
    let balance_id = get_balance_contract_id(provider, wallet).await;
    let amount = 55u64;

    let asset_id = testfuelcoincontract_mod::ContractId {
        value: fuelcoin_id.into(),
    };

    let target = testfuelcoincontract_mod::ContractId {
        value: balance_id.into(),
    };

    fuelcoin_instance
        .mint_and_send_to_contract(amount, target.clone())
        .set_contracts(&[balance_id])
        .call()
        .await
        .unwrap();

    let result = fuelcoin_instance
        .get_balance(asset_id, target)
        .set_contracts(&[balance_id])
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, amount)
}

async fn can_mint_and_send_to_address() {
    let (provider, wallet) = setup_test_provider_and_wallet().await;
    let (fuelcoin_instance, fuelcoin_id) =
        get_fuelcoin_instance(provider.clone(), wallet.clone()).await;
    let balance_id = get_balance_contract_id(provider, wallet.clone()).await;
    let amount = 55u64;

    let asset_id_array: [u8; 32] = fuelcoin_id.into();

    let address = wallet.address();
    let recipient = testfuelcoincontract_mod::Address {
        value: address.into(),
    };

    fuelcoin_instance
        .mint_and_send_to_address(amount, recipient)
        .set_contracts(&[balance_id])
        .call()
        .await
        .unwrap();

    assert_eq!(wallet
        .get_spendable_coins(&AssetId::from(asset_id_array), 1)
        .await
        .unwrap()[0]
        .amount, amount.into()
    );
}

async fn get_fuelcoin_instance(
    provider: Provider,
    wallet: Wallet,
) -> (TestFuelCoinContract, ContractId) {
    let salt = Salt::from([0u8; 32]);
    let compiled =
        Contract::load_sway_contract("test_projects/token_ops/out/debug/token_ops.bin", salt)
            .unwrap();
    let fuelcoin_id = Contract::deploy(&compiled, &provider, &wallet, TxParameters::default())
        .await
        .unwrap();

    let fuelcoin_instance = TestFuelCoinContract::new(fuelcoin_id.to_string(), provider, wallet);

    (fuelcoin_instance, fuelcoin_id)
}

async fn get_balance_contract_id(provider: Provider, wallet: Wallet) -> ContractId {
    let salt = Salt::from([0u8; 32]);
    let compiled = Contract::load_sway_contract(
        "test_artifacts/balance_contract/out/debug/balance_contract.bin",
        salt,
    )
    .unwrap();
    let balance_id = Contract::deploy(&compiled, &provider, &wallet, TxParameters::default())
        .await
        .unwrap();

    balance_id
}
