use fuel_tx::{ContractId, Salt};
use fuels_abigen_macro::abigen;
use fuels_contract::{contract::Contract, parameters::TxParameters};
use fuels_signers::{util::test_helpers, LocalWallet};

abigen!(Escrow, "out/debug/escrow-abi.json");

// TODO: if contract storage is exposed then testing should be updated to validate state instead of only the return from a function
// TODO: update tests to reflect contract

struct Metadata {
    contract: Escrow,
    wallet: LocalWallet
}

async fn setup() -> (Escrow, Escrow, Escrow, LocalWallet, LocalWallet, LocalWallet) {
    // Deploy the compiled contract
    let salt = Salt::from([0u8; 32]);
    let compiled = Contract::load_sway_contract("./out/debug/escrow.bin", salt).unwrap();

    // Launch a local network and deploy the contract
    let (provider, deployer_wallet) = test_helpers::setup_test_provider_and_wallet().await;
    let (_, buyer_wallet) = test_helpers::setup_test_provider_and_wallet().await;
    let (_, seller_wallet) = test_helpers::setup_test_provider_and_wallet().await;

    let id = Contract::deploy(&compiled, &provider, &deployer_wallet, TxParameters::default())
        .await
        .unwrap();
    
    let deployer = Metadata {
        contract: Escrow::new(id.to_string(), &provider, &deployer_wallet),
        wallet: deployer_wallet
    }

    let buyer = Metadata {
        contract: Escrow::new(id.to_string(), &provider, &buyer_wallet),
        wallet: buyer_wallet
    }

    let seller = Metadata {
        contract: Escrow::new(id.to_string(), &provider, &seller_wallet),
        wallet: seller_wallet
    }

    (deployer, buyer, seller)
}

#[tokio::test]
async fn constructor() {
    let price: u64 = 100;
    let (deployer, buyer, seller) = setup().await;
    
    assert!(deployer.contract.constructor(buyer.address, seller.address, price).call().await.unwrap().value);
}

#[tokio::test]
async fn deposit() {
    let price: u64 = 100;
    let (deployer, buyer, seller) = setup().await;

    // Calldata
    let gas = 5000;
    let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
    // Init conditions
    assert!(deployer.contract.constructor(buyer.address, seller.address, &price).call().await.unwrap().value);

    // Test
    assert!(buyer.contract.deposit {gas, asset_id, amount: price} ().call().await.unwrap().value);
}

#[tokio::test]
#[should_panic(expected = "RESERV00")]
async fn deposit_panics_with_incorrect_state() {
    let price: u64 = 100;
    let (deployer, buyer, seller) = setup().await;

    // Specify calldata
    let gas = 5000;
    let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
    // Panic
    buyer.contract.deposit {gas, asset_id, amount: price} ().call().await.unwrap();
}

#[tokio::test]
#[should_panic(expected = "RESERV00")]
async fn deposit_panics_with_incorrect_asset_amount() {
    let price: u64 = 100;
    let (deployer, buyer, seller) = setup().await;

    // Calldata
    let gas = 5000;
    let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
    // Init conditions
    assert!(deployer.contract.constructor(buyer.address, seller.address, &price).call().await.unwrap().value);

    // Should panic
    buyer.contract.deposit {gas, asset_id, amount: price + 1} ().call().await.unwrap();
}

#[tokio::test]
#[should_panic(expected = "RESERV00")]
async fn deposit_panics_when_sender_is_not_the_correct_address() {
    let price: u64 = 100;
    let (deployer, buyer, seller) = setup().await;

    // Calldata
    let gas = 5000;
    let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
    // Init conditions
    assert!(deployer.contract.constructor(buyer.address, seller.address, &price).call().await.unwrap().value);

    // Should panic
    deployer.contract.deposit {gas, asset_id, amount: price + 1} ().call().await.unwrap();
}

#[tokio::test]
#[should_panic(expected = "RESERV00")]
async fn deposit_panics_when_already_deposited() {
    let price: u64 = 100;
    let (deployer, buyer, seller) = setup().await;

    // Calldata
    let gas = 5000;
    let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
    // Init conditions
    assert!(deployer.contract.constructor(buyer.address, seller.address, &price).call().await.unwrap().value);
    assert!(buyer.contract.deposit {gas, asset_id, amount: &price} ().call().await.unwrap().value);

    // Should panic
    buyer.contract.deposit {gas, asset_id, amount: price} ().call().await.unwrap();
}

#[tokio::test]
async fn approve() {
    // TODO: add transfer code into function and complete test by checking transfer?
    
    let price: u64 = 100;
    let (deployer, buyer, seller) = setup().await;

    // Calldata
    let gas = 5000;
    let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
    // Init conditions
    assert!(deployer.contract.constructor(buyer.address, seller.address, &price).call().await.unwrap().value);
    assert!(buyer.contract.deposit {gas, asset_id, amount: &price} ().call().await.unwrap().value);
    assert!(seller.contract.deposit {gas, asset_id, amount: &price} ().call().await.unwrap().value);

    // Test
    assert!(buyer.contract.approve().call().await.unwrap().value);
    assert!(seller.contract.approve().call().await.unwrap().value);
}

#[tokio::test]
#[should_panic(expected = "RESERV00")]
async fn approve_panics_with_incorrect_state() {
    let price: u64 = 100;
    let (deployer, buyer, seller) = setup().await;

    // Specify calldata
    let gas = 5000;
    let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
    // Panic
    buyer.contract.approve {gas, asset_id, amount: price} ().call().await.unwrap();
}

#[tokio::test]
#[should_panic(expected = "RESERV00")]
async fn approve_panics_when_sender_is_not_the_correct_address() {
    let price: u64 = 100;
    let (deployer, buyer, seller) = setup().await;

    // Calldata
    let gas = 5000;
    let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
    // Init conditions
    assert!(deployer.contract.constructor(buyer.address, seller.address, &price).call().await.unwrap().value);
    // Can add deposit assertion here, not neccessary though

    // Should panic
    deployer.contract.approve {gas, asset_id, amount: price + 1} ().call().await.unwrap();
}

#[tokio::test]
#[should_panic(expected = "RESERV00")]
async fn approve_panics_when_not_deposited() {
    let price: u64 = 100;
    let (deployer, buyer, seller) = setup().await;

    // Calldata
    let gas = 5000;
    let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
    // Init conditions
    assert!(deployer.contract.constructor(buyer.address, seller.address, &price).call().await.unwrap().value);

    // Should panic
    buyer.contract.approve {gas, asset_id, amount: price} ().call().await.unwrap();
}

#[tokio::test]
#[should_panic(expected = "RESERV00")]
async fn approve_panics_when_already_approved() {
    let price: u64 = 100;
    let (deployer, buyer, seller) = setup().await;

    // Calldata
    let gas = 5000;
    let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
    // Init conditions
    assert!(deployer.contract.constructor(buyer.address, seller.address, &price).call().await.unwrap().value);
    assert!(buyer.contract.deposit {gas, asset_id, amount: &price} ().call().await.unwrap().value);
    assert!(buyer.contract.approve {gas, asset_id, amount: price} ().call().await.unwrap().value);

    // Should panic
    buyer.contract.approve {gas, asset_id, amount: price} ().call().await.unwrap();
}

#[tokio::test]
async fn withdraw() {
    // TODO: add transfer code into function and complete test by checking transfer?
    
    let price: u64 = 100;
    let (deployer, buyer, seller) = setup().await;

    // Calldata
    let gas = 5000;
    let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
    // Init conditions
    assert!(deployer.contract.constructor(buyer.address, seller.address, &price).call().await.unwrap().value);
    assert!(buyer.contract.deposit {gas, asset_id, amount: &price} ().call().await.unwrap().value);
    // Can add approve assertion here, not neccessary though

    // Test
    assert!(buyer.contract.withdraw().call().await.unwrap().value);
}

#[tokio::test]
#[should_panic(expected = "RESERV00")]
async fn withdraw_panics_with_incorrect_state() {
    let price: u64 = 100;
    let (deployer, buyer, seller) = setup().await;

    // Specify calldata
    let gas = 5000;
    let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
    // Panic
    buyer.contract.withdraw {gas, asset_id, amount: price} ().call().await.unwrap();
}

#[tokio::test]
#[should_panic(expected = "RESERV00")]
async fn withdraw_panics_when_sender_is_not_the_correct_address() {
    let price: u64 = 100;
    let (deployer, buyer, seller) = setup().await;

    // Calldata
    let gas = 5000;
    let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
    // Init conditions
    assert!(deployer.contract.constructor(buyer.address, seller.address, &price).call().await.unwrap().value);
    // Can add deposit assertion here, not neccessary though

    // Should panic
    deployer.contract.withdraw {gas, asset_id, amount: price} ().call().await.unwrap();
}

#[tokio::test]
#[should_panic(expected = "RESERV00")]
async fn withdraw_panics_when_not_deposited() {
    let price: u64 = 100;
    let (deployer, buyer, seller) = setup().await;

    // Calldata
    let gas = 5000;
    let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    
    // Init conditions
    assert!(deployer.contract.constructor(buyer.address, seller.address, &price).call().await.unwrap().value);

    // Should panic
    buyer.contract.withdraw {gas, asset_id, amount: price} ().call().await.unwrap();
}