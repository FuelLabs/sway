use fuels::prelude::*;

// Load abi from json
abigen!(Script(
    name = "MyScript",
    abi = "out/debug/{{project-name}}-abi.json"
));

async fn get_script_instance() -> MyScript<WalletUnlocked> {
    // Launch a local network
    let wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(1),             /* Single wallet */
            Some(1),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        None,
        None,
    )
    .await;
    let wallet = wallets.unwrap().pop().unwrap();

    let bin_path = "./out/debug/{{project-name}}.bin";

    let instance = MyScript::new(wallet.clone(), bin_path);

    instance
}

#[tokio::test]
async fn can_get_script_instance() {
    const LUCKY_NUMBER: u64 = 777; 
    let configurables = MyScriptConfigurables::default().with_SECRET_NUMBER(LUCKY_NUMBER.clone()).unwrap();
    
    let instance = get_script_instance().await;
    
    // Now you have an instance of your script
    let response = instance.with_configurables(configurables).main().call().await.unwrap();

    assert_eq!(response.value, LUCKY_NUMBER);

    // You can print logs from scripts to debug
    let logs = response.decode_logs();
    println!("{:?}", logs);
}
