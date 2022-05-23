use test_helpers::script_runner;
use fuel_tx::ContractId;
use fuels::prelude::*;
use fuels_abigen_macro::abigen;
use fuels_signers::wallet::Wallet;

abigen!(
    TestPowContract,
    "test_artifacts/pow/out/debug/pow-abi.json"
);

#[tokio::test]
async fn can_perform_exponentiation() {
    let path_to_bin = "test_projects/exponentiation/out/debug/exponentiation.bin";
    let return_val = script_runner(path_to_bin).await;
    assert_eq!(return_val, 1);
}

#[tokio::test]
#[should_panic(expected = "ArithmeticOverflow")]
async fn overflowing_pow_u64_panics() {
    let wallet = launch_provider_and_get_wallet().await;
    let (pow_instance, _) = get_pow_test_instance(wallet).await;
    let result = pow_instance.u64_overflow(100u64, 100u64)
        .call()
        .await
        .unwrap();
    dbg!(&result);

    // assert_eq!(result.value, 100);
}

#[tokio::test]
#[should_panic(expected = "ArithmeticOverflow")]
async fn overflowing_pow_u32_panics() {
    let wallet = launch_provider_and_get_wallet().await;
    let (pow_instance, _) = get_pow_test_instance(wallet).await;
    pow_instance.u32_overflow(10u32, 11u32)
        .call()
        .await
        .unwrap();
}

#[tokio::test]
#[should_panic(expected = "ArithmeticOverflow")]
async fn overflowing_pow_u16_panics() {
    let wallet = launch_provider_and_get_wallet().await;
    let (pow_instance, _) = get_pow_test_instance(wallet).await;
    pow_instance.u16_overflow(10u16, 5u16)
        .call()
        .await
        .unwrap();
}

#[tokio::test]
#[should_panic(expected = "ArithmeticOverflow")]
async fn overflowing_pow_u8_panics() {
    let wallet = launch_provider_and_get_wallet().await;
    let (pow_instance, _) = get_pow_test_instance(wallet).await;
    pow_instance.u8_overflow(10u8, 3u8)
        .call()
        .await
        .unwrap();
}

async fn get_pow_test_instance(wallet: Wallet) -> (TestPowContract, ContractId) {
    let pow_id = Contract::deploy(
        "test_artifacts/pow/out/debug/pow.bin",
        &wallet,
        TxParameters::default(),
    )
    .await
    .unwrap();

    let pow_instance = TestPowContract::new(pow_id.to_string(), wallet);

    (pow_instance, pow_id)
}
