use fuels::{prelude::*, tx::ConsensusParameters};
use hex;

#[tokio::test]
async fn run_valid() -> Result<()> {
    let wallet = launch_provider_and_get_wallet().await;

    let mut tx = ScriptTransaction::new(
        vec![],
        vec![],
        TxParameters::new(
            None,
            Some(ConsensusParameters::DEFAULT.max_gas_per_tx),
            None,
        ),
    )
    .with_script(std::fs::read(
        "test_projects/logging/out/debug/logging.bin",
    )?);

    wallet.sign_transaction(&mut tx).await?;

    let receipts = wallet.get_provider()?.send_transaction(&tx).await?;

    let correct_hex =
        hex::decode("ef86afa9696cf0dc6385e2c407a6e159a1103cefb7e2ae0636fb33d3cb2a9e4a");

    assert_eq!(correct_hex.unwrap(), receipts[0].data().unwrap());

    Ok(())
}
