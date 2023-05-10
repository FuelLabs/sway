use fuels::prelude::*;
use hex;

#[tokio::test]
async fn run_valid() -> Result<()> {
    abigen!(Script(
        name = "Logging",
        abi = "test_projects/logging/out/debug/logging-abi.json",
    ));

    let wallet = launch_provider_and_get_wallet().await;
    let bin_path = "test_projects/logging/out/debug/logging.bin";
    let instance = Logging::new(wallet.clone(), bin_path);

    let response = instance.main().call().await?;

    let correct_hex =
        hex::decode("ef86afa9696cf0dc6385e2c407a6e159a1103cefb7e2ae0636fb33d3cb2a9e4a");

    assert_eq!(correct_hex.unwrap(), response.receipts[0].data().unwrap());

    Ok(())
}
