use fuels::{
    accounts::wallet::WalletUnlocked,
    prelude::*,
    types::U256 as FU256,
};
use ruint::aliases::U256 as RU256;

abigen!(Contract(
    name = "U256LogTestContract",
    abi = "test_projects/u256_log/out/release/u256_log-abi.json"
));

async fn get_u256_log_instance() -> (U256LogTestContract<WalletUnlocked>, ContractId) {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "test_projects/u256_log/out/release/u256_log.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap();
    let instance = U256LogTestContract::new(id.clone(), wallet);

    (instance, id.into())
}

#[tokio::test]
async fn test_parity() {
    let (instance, _id) = get_u256_log_instance().await;
    let contract_methods = instance.methods();

    for a in 1..=100 {
        for b in 2..=100 {
            let result = contract_methods.u256_log(FU256::from(a), FU256::from(b)).call().await.unwrap().value.as_usize();
            let expected = RU256::from(a).log(RU256::from(b));
            assert_eq!(result, expected, "a: {}, b: {}, result: {}, expected: {}", a, b, result, expected);
        }
    }
}

// #[tokio::test]
// async fn test_log2() {
//     let (instance, _id) = get_u256_log_instance().await;
//     let contract_methods = instance.methods();

//     for a in 1..=100 {
//         let result = contract_methods.u256_log2(FU256::from(a)).call().await.unwrap().value.as_usize();
//         let expected = RU256::from(a).log2();
//         assert_eq!(result, expected, "a: {}, result: {}, expected: {}", a, result, expected);
//     }
// }