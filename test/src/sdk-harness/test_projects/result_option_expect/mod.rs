use fuels::{accounts::wallet::WalletUnlocked, prelude::*, types::Bits256};
use std::str::FromStr;

abigen!(Contract(
    name = "ExpectTestingContract",
    abi = "test_projects/result_option_expect/out/debug/result_option_expect-abi.json"
));

async fn setup() -> (ExpectTestingContract<WalletUnlocked>, ContractId) {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "test_projects/result_option_expect/out/debug/result_option_expect.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap();
    let instance = ExpectTestingContract::new(id.clone(), wallet);

    (instance, id.into())
}
