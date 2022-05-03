use fuels::{
    prelude::{Contract, TxParameters, LocalWallet, CallParameters},
    signers::Signer,
    test_helpers,
};
use fuels_abigen_macro::abigen;
use fuel_tx::{ContractId, Salt, AssetId};

abigen!(Escrow, "out/debug/escrow-abi.json");
abigen!(Asset, "tests/artifacts/asset/out/debug/asset-abi.json");

// TODO: if contract storage is exposed then testing should be updated to validate state instead of only the return from a function
// TODO: update tests to reflect contract

struct Metadata {
    escrow: Escrow,
    asset: Option<Asset>,
    wallet: LocalWallet
}

async fn setup() -> (Metadata, Metadata, Metadata, ContractId) {
    // Deploy the compiled contract
    let salt = Salt::from([0u8; 32]);
    let compiled_escrow = Contract::load_sway_contract("./out/debug/escrow.bin", salt).unwrap();
    let compiled_asset = Contract::load_sway_contract("./tests/artifacts/asset/out/debug/asset.bin", salt).unwrap();

    // Launch a local network and deploy the contract
    let (provider, deployer_wallet) = test_helpers::setup_test_provider_and_wallet().await;
    let (_, buyer_wallet) = test_helpers::setup_test_provider_and_wallet().await;
    let (_, seller_wallet) = test_helpers::setup_test_provider_and_wallet().await;

    let escrow_id = Contract::deploy(&compiled_escrow, &provider, &deployer_wallet, TxParameters::default())
        .await
        .unwrap();
    
    let asset_id = Contract::deploy(&compiled_asset, &provider, &deployer_wallet, TxParameters::default())
        .await
        .unwrap();

    let deployer = Metadata {
        escrow: Escrow::new(escrow_id.to_string(), provider.clone(), deployer_wallet.clone()),
        asset: Some(Asset::new(asset_id.to_string(), provider.clone(), deployer_wallet.clone())),
        wallet: deployer_wallet.clone()
    };

    let buyer = Metadata {
        escrow: Escrow::new(escrow_id.to_string(), provider.clone(), buyer_wallet.clone()),
        asset: None,
        wallet: buyer_wallet
    };

    let seller = Metadata {
        escrow: Escrow::new(escrow_id.to_string(), provider.clone(), seller_wallet.clone()),
        asset: None,
        wallet: seller_wallet
    };

    (deployer, buyer, seller, asset_id)
}

#[tokio::test]
async fn constructor() {
    let amount: u64 = 100;
    let (deployer, buyer, seller, asset_id) = setup().await;
    
    assert!(deployer.escrow.constructor(buyer.wallet.address(), seller.wallet.address(), asset_id, amount).call().await.unwrap().value);
}

#[tokio::test]
async fn deposit() {
    let amount: u64 = 100;
    let (deployer, buyer, seller, asset_id) = setup().await;
    
    // Init conditions
    assert!(deployer.escrow.constructor(buyer.wallet.address(), seller.wallet.address(), asset_id, amount).call().await.unwrap().value);
    assert!(deployer.asset.unwrap().mint_and_send_to_address(amount, buyer.wallet.address()).append_variable_outputs(1).call().await.unwrap().value);

    // Test
    let tx_params = TxParameters::new(None, Some(1_000_000), None, None);    
    let call_params = CallParameters::new(Some(amount), Some(AssetId::from(*asset_id)));

    assert!(buyer.escrow.deposit().tx_params(tx_params).call_params(call_params).call().await.unwrap().value);
}

// #[tokio::test]
// #[should_panic(expected = "RESERV00")]
// async fn deposit_panics_with_incorrect_state() {
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Specify calldata
//     let gas = 5000;
    
//     // Panic
//     buyer.escrow.deposit {gas, asset_id, amount} ().call().await.unwrap();
// }

// #[tokio::test]
// #[should_panic(expected = "RESERV00")]
// async fn deposit_panics_with_incorrect_asset_amount() {
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Calldata
//     let gas = 5000;
    
//     // Init conditions
//     assert!(deployer.escrow.constructor(buyer.wallet.address(), seller.wallet.address(), asset_id, amount).call().await.unwrap().value);

//     // Should panic
//     buyer.escrow.deposit {gas, asset_id, amount: amount + 1} ().call().await.unwrap();
// }

// #[tokio::test]
// #[should_panic(expected = "RESERV00")]
// async fn deposit_panics_when_sender_is_not_the_correct_address() {
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Calldata
//     let gas = 5000;
    
//     // Init conditions
//     assert!(deployer.escrow.constructor(buyer.wallet.address(), seller.wallet.address(), asset_id, amount).call().await.unwrap().value);

//     // Should panic
//     deployer.escrow.deposit {gas, asset_id, amount} ().call().await.unwrap();
// }

// #[tokio::test]
// #[should_panic(expected = "RESERV00")]
// async fn deposit_panics_when_already_deposited() {
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Calldata
//     let gas = 5000;
    
//     // Init conditions
//     assert!(deployer.escrow.constructor(buyer.wallet.address(), seller.wallet.address(), asset_id, amount).call().await.unwrap().value);
//     assert!(buyer.escrow.deposit {gas, asset_id, amount: &amount} ().call().await.unwrap().value);

//     // Should panic
//     buyer.escrow.deposit {gas, asset_id, amount} ().call().await.unwrap();
// }

// #[tokio::test]
// async fn approve() {
    
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Calldata
//     let gas = 5000;
    
//     // Init conditions
//     assert!(deployer.escrow.constructor(buyer.wallet.address(), seller.wallet.address(), asset_id, amount).call().await.unwrap().value);//     assert!(buyer.escrow.deposit {gas, asset_id, amount: &amount} ().call().await.unwrap().value);
//     assert!(seller.escrow.deposit {gas, asset_id, amount: &amount} ().call().await.unwrap().value);

//     // Test
//     assert!(buyer.escrow.approve().call().await.unwrap().value);
//     assert!(seller.escrow.approve().call().await.unwrap().value);
// }

// #[tokio::test]
// #[should_panic(expected = "RESERV00")]
// async fn approve_panics_with_incorrect_state() {
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Specify calldata
//     let gas = 5000;
    
//     // Panic
//     buyer.escrow.approve {gas, asset_id, amount} ().call().await.unwrap();
// }

// #[tokio::test]
// #[should_panic(expected = "RESERV00")]
// async fn approve_panics_when_sender_is_not_the_correct_address() {
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Calldata
//     let gas = 5000;
    
//     // Init conditions
//     assert!(deployer.escrow.constructor(buyer.wallet.address(), seller.wallet.address(), asset_id, amount).call().await.unwrap().value);
//     // Can add deposit assertion here, not neccessary though

//     // Should panic
//     deployer.escrow.approve {gas, asset_id, amount} ().call().await.unwrap();
// }

// #[tokio::test]
// #[should_panic(expected = "RESERV00")]
// async fn approve_panics_when_not_deposited() {
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Calldata
//     let gas = 5000;
    
//     // Init conditions
//     assert!(deployer.escrow.constructor(buyer.wallet.address(), seller.wallet.address(), asset_id, amount).call().await.unwrap().value);

//     // Should panic
//     buyer.escrow.approve {gas, asset_id, amount} ().call().await.unwrap();
// }

// #[tokio::test]
// #[should_panic(expected = "RESERV00")]
// async fn approve_panics_when_already_approved() {
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Calldata
//     let gas = 5000;
    
//     // Init conditions
//     assert!(deployer.escrow.constructor(buyer.wallet.address(), seller.wallet.address(), asset_id, amount).call().await.unwrap().value);//     assert!(buyer.escrow.deposit {gas, asset_id, amount: &amount} ().call().await.unwrap().value);
//     assert!(buyer.escrow.approve {gas, asset_id, amount: amount} ().call().await.unwrap().value);

//     // Should panic
//     buyer.escrow.approve {gas, asset_id, amount} ().call().await.unwrap();
// }

// #[tokio::test]
// async fn withdraw() {
//     // TODO: add transfer code into function and complete test by checking transfer?
    
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Calldata
//     let gas = 5000;
    
//     // Init conditions
//     assert!(deployer.escrow.constructor(buyer.wallet.address(), seller.wallet.address(), asset_id, amount).call().await.unwrap().value);
//     assert!(buyer.escrow.deposit {gas, asset_id, amount: &amount} ().call().await.unwrap().value);
//     // Can add approve assertion here, not neccessary though

//     // Test
//     assert!(buyer.escrow.withdraw().call().await.unwrap().value);
// }

// #[tokio::test]
// #[should_panic(expected = "RESERV00")]
// async fn withdraw_panics_with_incorrect_state() {
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Specify calldata
//     let gas = 5000;
    
//     // Panic
//     buyer.escrow.withdraw {gas, asset_id, amount} ().call().await.unwrap();
// }

// #[tokio::test]
// #[should_panic(expected = "RESERV00")]
// async fn withdraw_panics_when_sender_is_not_the_correct_address() {
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Calldata
//     let gas = 5000;
    
//     // Init conditions
//     assert!(deployer.escrow.constructor(buyer.wallet.address(), seller.wallet.address(), asset_id, amount).call().await.unwrap().value);
//     // Can add deposit assertion here, not neccessary though

//     // Should panic
//     deployer.escrow.withdraw {gas, asset_id, amount} ().call().await.unwrap();
// }

// #[tokio::test]
// #[should_panic(expected = "RESERV00")]
// async fn withdraw_panics_when_not_deposited() {
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Calldata
//     let gas = 5000;
    
//     // Init conditions
//     assert!(deployer.escrow.constructor(buyer.wallet.address(), seller.wallet.address(), asset_id, amount).call().await.unwrap().value);

//     // Should panic
//     buyer.escrow.withdraw {gas, asset_id, amount} ().call().await.unwrap();
// }
