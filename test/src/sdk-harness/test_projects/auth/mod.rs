use fuels::{
    accounts::wallet::{Wallet, WalletUnlocked},
    prelude::*,
    types::ContractId,
};
use fuel_core_client::client::types::primitives::ChainId;
use fuel_vm::fuel_tx::{field::*, Input as TxInput};

abigen!(
    Contract(
        name = "AuthContract",
        abi = "test_artifacts/auth_testing_contract/out/debug/auth_testing_contract-abi.json"
    ),
    Contract(
        name = "AuthCallerContract",
        abi = "test_artifacts/auth_caller_contract/out/debug/auth_caller_contract-abi.json"
    )
);

#[tokio::test]
async fn is_external_from_sdk() {
    let (auth_instance, _, _, _, _) = get_contracts().await;
    let result = auth_instance
        .methods()
        .is_caller_external()
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, true);
}

#[tokio::test]
async fn msg_sender_from_sdk() {
    let (auth_instance, _, _, _, wallet) = get_contracts().await;
    let result = auth_instance
        .methods()
        .returns_msg_sender_address(wallet.address())
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, true);
}

#[tokio::test]
async fn msg_sender_from_contract() {
    let (auth_instance, auth_id, caller_instance, caller_id, _) = get_contracts().await;

    let result = caller_instance
        .methods()
        .call_auth_contract(auth_id, caller_id)
        .set_contracts(&[&auth_instance])
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, true);
}


#[tokio::test]
async fn msg_sender_message() {
    let (auth_instance, _, _, _, wallet) = get_contracts().await;

    const MESSAGE_DATA: [u8; 3] = [1u8, 2u8, 3u8];
    let mut wallet = WalletUnlocked::new_random(None);
    let mut coins = setup_single_asset_coins(wallet.address(), BASE_ASSET_ID, 100, 1000);
    let messages = setup_single_message(
        &Bech32Address {
            hrp: "".to_string(),
            hash: Default::default(),
        },
        wallet.address(),
        DEFAULT_COIN_AMOUNT,
        0.into(),
        MESSAGE_DATA.to_vec(),
    );
    let (provider, _address) = setup_test_provider(coins.clone(), vec![messages], None, None).await;
    wallet.set_provider(provider.clone());

    let contract_id = Contract::load_from(
        "test_artifacts/auth_testing_contract/out/debug/auth_testing_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxParameters::default())
    .await
    .unwrap();

    let instance = AuthContract::new(contract_id.clone(), wallet.clone());

    let handler = auth_instance.methods().returns_msg_sender_address(wallet.address());
    let mut tx = handler.build_tx().await.unwrap();
    add_message_input(&mut tx, wallet.clone()).await;
    tx.precompute(*ChainId::default()).unwrap();

    let receipts = wallet
        .provider()
        .unwrap()
        .send_transaction(&tx)
        .await
        .unwrap();

    // true = 1
    assert_eq!(*receipts[1].data().unwrap(), [1u8]);
}

async fn add_message_input(tx: &mut ScriptTransaction, wallet: WalletUnlocked) {
    let message = &wallet.get_messages().await.unwrap()[0];

    let message_input = TxInput::message_data_signed(
        message.sender.clone().into(),
        message.recipient.clone().into(),
        message.amount,
        message.nonce,
        0,
        message.data.clone(),
    );

    tx.tx.inputs_mut().push(message_input);
}

async fn get_contracts() -> (
    AuthContract<WalletUnlocked>,
    ContractId,
    AuthCallerContract<WalletUnlocked>,
    ContractId,
    Wallet,
) {
    let wallet = launch_provider_and_get_wallet().await;

    let id_1 = Contract::load_from(
        "test_artifacts/auth_testing_contract/out/debug/auth_testing_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxParameters::default())
    .await
    .unwrap();

    let id_2 = Contract::load_from(
        "test_artifacts/auth_caller_contract/out/debug/auth_caller_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxParameters::default())
    .await
    .unwrap();

    let instance_1 = AuthContract::new(id_1.clone(), wallet.clone());
    let instance_2 = AuthCallerContract::new(id_2.clone(), wallet.clone());

    (
        instance_1,
        id_1.into(),
        instance_2,
        id_2.into(),
        wallet.lock(),
    )
}
