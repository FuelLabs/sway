use fuel_tx::{ContractId, Salt};
use fuels_abigen_macro::abigen;
use fuels_contract::{contract::Contract, parameters::TxParameters};
use fuels_signers::{util::test_helpers, LocalWallet, Signer};

abigen!(Escrow, "out/debug/escrow-abi.json");
abigen!(Asset, "tests/artifacts/asset/out/debug/asset-abi.json");

// TODO: if contract storage is exposed then testing should be updated to validate state instead of only the return from a function
// TODO: update tests to reflect contract

struct Metadata {
    contract: Escrow,
    wallet: LocalWallet
}

async fn setup() -> (Metadata, Metadata, Metadata, asset_mod::Asset, ContractId) {
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
        contract: Escrow::new(escrow_id.to_string(), provider.clone(), deployer_wallet.clone()),
        wallet: deployer_wallet.clone()
    };

    let buyer = Metadata {
        contract: Escrow::new(escrow_id.to_string(), provider.clone(), buyer_wallet.clone()),
        wallet: buyer_wallet
    };

    let seller = Metadata {
        contract: Escrow::new(escrow_id.to_string(), provider.clone(), seller_wallet.clone()),
        wallet: seller_wallet
    };

    let asset = Asset::new(asset_id.to_string(), provider.clone(), deployer_wallet.clone());

    (deployer, buyer, seller, asset, asset_id)
}

#[tokio::test]
async fn constructor() {
    let amount: u64 = 100;
    let (deployer, buyer, seller, _, asset_id) = setup().await;

    let thing = deployer.contract.constructor(buyer.wallet.address(), seller.wallet.address(), asset_id, amount).call().await;
    println!("{:#?}", thing);
    
    assert!(thing.unwrap().value);
}

// #[tokio::test]
// async fn deposit() {
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Calldata
//     let gas = 5000;
//     let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
//     // Init conditions
//     assert!(deployer.contract.constructor(buyer.address, seller.address, &amount).call().await.unwrap().value);

//     // Test
//     assert!(buyer.contract.deposit {gas, asset_id, amount} ().call().await.unwrap().value);
// }

// #[tokio::test]
// #[should_panic(expected = "RESERV00")]
// async fn deposit_panics_with_incorrect_state() {
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Specify calldata
//     let gas = 5000;
//     let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
//     // Panic
//     buyer.contract.deposit {gas, asset_id, amount} ().call().await.unwrap();
// }

// #[tokio::test]
// #[should_panic(expected = "RESERV00")]
// async fn deposit_panics_with_incorrect_asset_amount() {
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Calldata
//     let gas = 5000;
//     let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
//     // Init conditions
//     assert!(deployer.contract.constructor(buyer.address, seller.address, &amount).call().await.unwrap().value);

//     // Should panic
//     buyer.contract.deposit {gas, asset_id, amount: amount + 1} ().call().await.unwrap();
// }

// #[tokio::test]
// #[should_panic(expected = "RESERV00")]
// async fn deposit_panics_when_sender_is_not_the_correct_address() {
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Calldata
//     let gas = 5000;
//     let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
//     // Init conditions
//     assert!(deployer.contract.constructor(buyer.address, seller.address, &amount).call().await.unwrap().value);

//     // Should panic
//     deployer.contract.deposit {gas, asset_id, amount} ().call().await.unwrap();
// }

// #[tokio::test]
// #[should_panic(expected = "RESERV00")]
// async fn deposit_panics_when_already_deposited() {
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Calldata
//     let gas = 5000;
//     let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
//     // Init conditions
//     assert!(deployer.contract.constructor(buyer.address, seller.address, &amount).call().await.unwrap().value);
//     assert!(buyer.contract.deposit {gas, asset_id, amount: &amount} ().call().await.unwrap().value);

//     // Should panic
//     buyer.contract.deposit {gas, asset_id, amount} ().call().await.unwrap();
// }

// #[tokio::test]
// async fn approve() {
//     // TODO: add transfer code into function and complete test by checking transfer?
    
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Calldata
//     let gas = 5000;
//     let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
//     // Init conditions
//     assert!(deployer.contract.constructor(buyer.address, seller.address, &amount).call().await.unwrap().value);
//     assert!(buyer.contract.deposit {gas, asset_id, amount: &amount} ().call().await.unwrap().value);
//     assert!(seller.contract.deposit {gas, asset_id, amount: &amount} ().call().await.unwrap().value);

//     // Test
//     assert!(buyer.contract.approve().call().await.unwrap().value);
//     assert!(seller.contract.approve().call().await.unwrap().value);
// }

// #[tokio::test]
// #[should_panic(expected = "RESERV00")]
// async fn approve_panics_with_incorrect_state() {
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Specify calldata
//     let gas = 5000;
//     let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
//     // Panic
//     buyer.contract.approve {gas, asset_id, amount} ().call().await.unwrap();
// }

// #[tokio::test]
// #[should_panic(expected = "RESERV00")]
// async fn approve_panics_when_sender_is_not_the_correct_address() {
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Calldata
//     let gas = 5000;
//     let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
//     // Init conditions
//     assert!(deployer.contract.constructor(buyer.address, seller.address, &amount).call().await.unwrap().value);
//     // Can add deposit assertion here, not neccessary though

//     // Should panic
//     deployer.contract.approve {gas, asset_id, amount} ().call().await.unwrap();
// }

// #[tokio::test]
// #[should_panic(expected = "RESERV00")]
// async fn approve_panics_when_not_deposited() {
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Calldata
//     let gas = 5000;
//     let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
//     // Init conditions
//     assert!(deployer.contract.constructor(buyer.address, seller.address, &amount).call().await.unwrap().value);

//     // Should panic
//     buyer.contract.approve {gas, asset_id, amount} ().call().await.unwrap();
// }

// #[tokio::test]
// #[should_panic(expected = "RESERV00")]
// async fn approve_panics_when_already_approved() {
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Calldata
//     let gas = 5000;
//     let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
//     // Init conditions
//     assert!(deployer.contract.constructor(buyer.address, seller.address, &amount).call().await.unwrap().value);
//     assert!(buyer.contract.deposit {gas, asset_id, amount: &amount} ().call().await.unwrap().value);
//     assert!(buyer.contract.approve {gas, asset_id, amount: amount} ().call().await.unwrap().value);

//     // Should panic
//     buyer.contract.approve {gas, asset_id, amount} ().call().await.unwrap();
// }

// #[tokio::test]
// async fn withdraw() {
//     // TODO: add transfer code into function and complete test by checking transfer?
    
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Calldata
//     let gas = 5000;
//     let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
//     // Init conditions
//     assert!(deployer.contract.constructor(buyer.address, seller.address, &amount).call().await.unwrap().value);
//     assert!(buyer.contract.deposit {gas, asset_id, amount: &amount} ().call().await.unwrap().value);
//     // Can add approve assertion here, not neccessary though

//     // Test
//     assert!(buyer.contract.withdraw().call().await.unwrap().value);
// }

// #[tokio::test]
// #[should_panic(expected = "RESERV00")]
// async fn withdraw_panics_with_incorrect_state() {
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Specify calldata
//     let gas = 5000;
//     let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
//     // Panic
//     buyer.contract.withdraw {gas, asset_id, amount} ().call().await.unwrap();
// }

// #[tokio::test]
// #[should_panic(expected = "RESERV00")]
// async fn withdraw_panics_when_sender_is_not_the_correct_address() {
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Calldata
//     let gas = 5000;
//     let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
//     // Init conditions
//     assert!(deployer.contract.constructor(buyer.address, seller.address, &amount).call().await.unwrap().value);
//     // Can add deposit assertion here, not neccessary though

//     // Should panic
//     deployer.contract.withdraw {gas, asset_id, amount} ().call().await.unwrap();
// }

// #[tokio::test]
// #[should_panic(expected = "RESERV00")]
// async fn withdraw_panics_when_not_deposited() {
//     let amount: u64 = 100;
//     let (deployer, buyer, seller) = setup().await;

//     // Calldata
//     let gas = 5000;
//     let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
//     // Init conditions
//     assert!(deployer.contract.constructor(buyer.address, seller.address, &amount).call().await.unwrap().value);

//     // Should panic
//     buyer.contract.withdraw {gas, asset_id, amount} ().call().await.unwrap();
// }