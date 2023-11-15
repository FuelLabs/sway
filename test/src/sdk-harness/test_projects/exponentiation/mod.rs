use fuels::accounts::wallet::WalletUnlocked;
use fuels::prelude::*;
use fuels::types::ContractId;

abigen!(Contract(
    name = "TestPowContract",
    abi = "test_artifacts/pow/out/debug/pow-abi.json"
));

#[tokio::test]
#[should_panic(expected = "ArithmeticOverflow")]
async fn overflowing_pow_u64_panics() {
    let wallet = launch_provider_and_get_wallet().await;
    let (pow_instance, _) = get_pow_test_instance(wallet).await;
    pow_instance
        .methods()
        .u64_overflow(100u64, 100u32)
        .call()
        .await
        .unwrap();
}

#[tokio::test]
// TODO won't overflow until https://github.com/FuelLabs/fuel-specs/issues/90 lands
// #[should_panic(expected = "ArithmeticOverflow")]
async fn overflowing_pow_u32_panics() {
    let wallet = launch_provider_and_get_wallet().await;
    let (pow_instance, _) = get_pow_test_instance(wallet).await;
    pow_instance
        .methods()
        .u32_overflow(10u32, 11u32)
        .call()
        .await
        .unwrap();
}

#[tokio::test]
#[should_panic(expected = "ArithmeticOverflow")]
async fn overflowing_pow_u32_panics_max() {
    let wallet = launch_provider_and_get_wallet().await;
    let (pow_instance, _) = get_pow_test_instance(wallet).await;
    pow_instance
        .methods()
        .u32_overflow(u32::MAX, u32::MAX)
        .call()
        .await
        .unwrap();
}

#[tokio::test]
// TODO won't overflow until https://github.com/FuelLabs/fuel-specs/issues/90 lands
// #[should_panic(expected = "ArithmeticOverflow")]
async fn overflowing_pow_u16_panics() {
    let wallet = launch_provider_and_get_wallet().await;
    let (pow_instance, _) = get_pow_test_instance(wallet).await;
    pow_instance
        .methods()
        .u16_overflow(10u16, 5u32)
        .call()
        .await
        .unwrap();
}

#[tokio::test]
#[should_panic(expected = "ArithmeticOverflow")]
async fn overflowing_pow_u16_panics_max() {
    let wallet = launch_provider_and_get_wallet().await;
    let (pow_instance, _) = get_pow_test_instance(wallet).await;
    pow_instance
        .methods()
        .u16_overflow(u16::MAX, u32::MAX)
        .call()
        .await
        .unwrap();
}

#[tokio::test]
// TODO won't overflow until https://github.com/FuelLabs/fuel-specs/issues/90 lands
// #[should_panic(expected = "ArithmeticOverflow")]
async fn overflowing_pow_u8_panics() {
    let wallet = launch_provider_and_get_wallet().await;
    let (pow_instance, _) = get_pow_test_instance(wallet).await;
    pow_instance.methods().u8_overflow(10u8, 3u32).call().await.unwrap();
}

#[tokio::test]
#[should_panic(expected = "ArithmeticOverflow")]
async fn overflowing_pow_u8_panics_max() {
    let wallet = launch_provider_and_get_wallet().await;
    let (pow_instance, _) = get_pow_test_instance(wallet).await;
    pow_instance
        .methods()
        .u8_overflow(u8::MAX, u32::MAX)
        .call()
        .await
        .unwrap();
}

async fn get_pow_test_instance(
    wallet: WalletUnlocked,
) -> (TestPowContract<WalletUnlocked>, ContractId) {
    let pow_id = Contract::load_from(
        "test_artifacts/pow/out/debug/pow.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxParameters::default())
    .await
    .unwrap();

    let pow_instance = TestPowContract::new(pow_id.clone(), wallet);

    (pow_instance, pow_id.into())
}
