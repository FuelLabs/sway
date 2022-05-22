// Uncomment when https://github.com/FuelLabs/fuels-rs/pull/305 (deploy_with_salt) lands in a new release
// use fuel_tx::{AssetId, ContractId, Salt};

use fuel_tx::{AssetId, ContractId};
use fuels::prelude::*;
use fuels_abigen_macro::abigen;

abigen!(Escrow, "out/debug/escrow-abi.json");
abigen!(Asset, "tests/artifacts/asset/out/debug/asset-abi.json");

struct Metadata {
    escrow: Escrow,
    asset: Option<Asset>,
    wallet: LocalWallet,
}

async fn setup() -> (Metadata, Metadata, Metadata, ContractId, u64) {
    // Create some addresses with the minimum amount of asset: 1 Million
    let (pk1, mut coins1) = setup_address_and_coins(1, 1000000);
    let (pk2, coins2) = setup_address_and_coins(1, 1000000);
    let (pk3, coins3) = setup_address_and_coins(1, 1000000);

    coins1.extend(coins2);
    coins1.extend(coins3);

    // Launch a provider with those coins
    let (provider, _) = setup_test_provider(coins1).await;

    // Get the wallets from that provider
    let deployer_wallet = LocalWallet::new_from_private_key(pk1, provider.clone());
    let buyer_wallet = LocalWallet::new_from_private_key(pk2, provider.clone());
    let seller_wallet = LocalWallet::new_from_private_key(pk3, provider);

    let escrow_id = Contract::deploy(
        "./out/debug/escrow.bin",
        &deployer_wallet,
        TxParameters::default(),
    )
    .await
    .unwrap();

    let asset_id = Contract::deploy(
        "./tests/artifacts/asset/out/debug/asset.bin",
        &deployer_wallet,
        TxParameters::default(),
    )
    .await
    .unwrap();

    let deployer = Metadata {
        escrow: Escrow::new(escrow_id.to_string(), deployer_wallet.clone()),
        asset: Some(Asset::new(asset_id.to_string(), deployer_wallet.clone())),
        wallet: deployer_wallet,
    };

    let buyer = Metadata {
        escrow: Escrow::new(escrow_id.to_string(), buyer_wallet.clone()),
        asset: None,
        wallet: buyer_wallet,
    };

    let seller = Metadata {
        escrow: Escrow::new(escrow_id.to_string(), seller_wallet.clone()),
        asset: None,
        wallet: seller_wallet,
    };

    let asset_amount: u64 = 100;

    (deployer, buyer, seller, asset_id, asset_amount)
}

async fn init(
    deployer: &Metadata,
    buyer: &LocalWallet,
    seller: &LocalWallet,
    asset_id: ContractId,
    asset_amount: u64,
) -> bool {
    deployer
        .escrow
        .constructor(buyer.address(), seller.address(), asset_id, asset_amount)
        .call()
        .await
        .unwrap()
        .value
}

async fn mint(deployer: &Metadata, user: &LocalWallet, asset_amount: u64) {
    deployer
        .asset
        .as_ref()
        .unwrap()
        .mint_and_send_to_address(asset_amount, user.address())
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap()
        .value;
}

async fn balance(escrow: &Escrow) -> u64 {
    escrow.get_balance().call().await.unwrap().value
}

async fn user_data(escrow: &Escrow, user: &LocalWallet) -> (bool, bool) {
    escrow
        .get_user_data(user.address())
        .call()
        .await
        .unwrap()
        .value
}

mod constructor {

    use super::*;

    #[tokio::test]
    async fn initializes() {
        let (deployer, buyer, seller, asset_id, asset_amount) = setup().await;

        assert!(
            init(
                &deployer,
                &buyer.wallet,
                &seller.wallet,
                asset_id,
                asset_amount
            )
            .await
        );
    }
}

mod deposit {

    use super::*;

    #[tokio::test]
    async fn deposits() {
        let (deployer, buyer, seller, asset_id, asset_amount) = setup().await;

        let tx_params = TxParameters::new(None, Some(1_000_000), None, None);
        let call_params = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));

        // Init conditions
        init(
            &deployer,
            &buyer.wallet,
            &seller.wallet,
            asset_id,
            asset_amount,
        )
        .await;
        mint(&deployer, &buyer.wallet, asset_amount).await;

        assert_eq!(balance(&deployer.escrow).await, 0);
        assert_eq!(
            user_data(&deployer.escrow, &buyer.wallet).await,
            (false, false)
        );

        // Test
        assert!(
            buyer
                .escrow
                .deposit()
                .tx_params(tx_params)
                .call_params(call_params)
                .call()
                .await
                .unwrap()
                .value
        );

        assert_eq!(balance(&deployer.escrow).await, asset_amount);
        assert_eq!(
            user_data(&deployer.escrow, &buyer.wallet).await,
            (true, false)
        );
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_when_not_initialized() {
        let (_, buyer, _, _, _) = setup().await;

        // Should panic
        buyer.escrow.deposit().call().await.unwrap();
    }

    // Uncomment when https://github.com/FuelLabs/fuels-rs/pull/305 (deploy_with_salt) lands in a new release
    // #[tokio::test]
    // #[should_panic(expected = "Revert(42)")]
    // async fn panics_with_incorrect_asset() {
    //     let (deployer, buyer, seller, asset_id, asset_amount) = setup().await;

    //     let another_asset_id = Contract::deploy_with_salt(
    //         "./tests/artifacts/asset/out/debug/asset.bin",
    //         &deployer.wallet,
    //         TxParameters::default(),
    //         Salt::from([1u8; 32]),
    //     )
    //     .await
    //     .unwrap();

    //     let another_asset = Asset::new(another_asset_id.to_string(), deployer.wallet.clone());

    //     let tx_params = TxParameters::new(None, Some(1_000_000), None, None);
    //     let call_params = CallParameters::new(Some(asset_amount), Some(AssetId::from(*another_asset_id)));

    //     // Init conditions
    //     init(
    //         &deployer,
    //         &buyer.wallet,
    //         &seller.wallet,
    //         asset_id,
    //         asset_amount,
    //     )
    //     .await;

    //     another_asset
    //         .mint_and_send_to_address(asset_amount, buyer.wallet.address())
    //         .append_variable_outputs(1)
    //         .call()
    //         .await
    //         .unwrap();

    //     // Should panic
    //     buyer
    //         .escrow
    //         .deposit()
    //         .tx_params(tx_params)
    //         .call_params(call_params)
    //         .call()
    //         .await
    //         .unwrap();
    // }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_with_incorrect_asset_amount() {
        let (deployer, buyer, seller, asset_id, asset_amount) = setup().await;

        let tx_params = TxParameters::new(None, Some(1_000_000), None, None);
        let call_params =
            CallParameters::new(Some(asset_amount - 1), Some(AssetId::from(*asset_id)));

        // Init conditions
        init(
            &deployer,
            &buyer.wallet,
            &seller.wallet,
            asset_id,
            asset_amount,
        )
        .await;
        mint(&deployer, &buyer.wallet, asset_amount).await;

        // Should panic
        buyer
            .escrow
            .deposit()
            .tx_params(tx_params)
            .call_params(call_params)
            .call()
            .await
            .unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_when_sender_is_not_the_correct_address() {
        let (deployer, buyer, seller, asset_id, asset_amount) = setup().await;

        let tx_params = TxParameters::new(None, Some(1_000_000), None, None);
        let call_params = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));

        // Init conditions
        init(
            &deployer,
            &buyer.wallet,
            &seller.wallet,
            asset_id,
            asset_amount,
        )
        .await;
        mint(&deployer, &deployer.wallet, asset_amount).await;

        // Should panic
        deployer
            .escrow
            .deposit()
            .tx_params(tx_params)
            .call_params(call_params)
            .call()
            .await
            .unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_when_already_deposited() {
        let (deployer, buyer, seller, asset_id, asset_amount) = setup().await;

        let tx_params1 = TxParameters::new(None, Some(1_000_000), None, None);
        let tx_params2 = TxParameters::new(None, Some(1_000_000), None, None);

        let call_params1 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));
        let call_params2 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));

        // Init conditions
        init(
            &deployer,
            &buyer.wallet,
            &seller.wallet,
            asset_id,
            asset_amount,
        )
        .await;
        mint(&deployer, &buyer.wallet, 2 * asset_amount).await;

        buyer
            .escrow
            .deposit()
            .tx_params(tx_params1)
            .call_params(call_params1)
            .call()
            .await
            .unwrap();

        // Should panic
        buyer
            .escrow
            .deposit()
            .tx_params(tx_params2)
            .call_params(call_params2)
            .call()
            .await
            .unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_after_both_parties_approve() {
        let (deployer, buyer, seller, asset_id, asset_amount) = setup().await;

        let tx_params1 = TxParameters::new(None, Some(1_000_000), None, None);
        let tx_params2 = TxParameters::new(None, Some(1_000_000), None, None);
        let tx_params3 = TxParameters::new(None, Some(1_000_000), None, None);

        let call_params1 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));
        let call_params2 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));
        let call_params3 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));

        // Init conditions
        init(
            &deployer,
            &buyer.wallet,
            &seller.wallet,
            asset_id,
            asset_amount,
        )
        .await;
        mint(&deployer, &buyer.wallet, asset_amount).await;
        mint(&deployer, &seller.wallet, asset_amount).await;

        buyer
            .escrow
            .deposit()
            .tx_params(tx_params1)
            .call_params(call_params1)
            .call()
            .await
            .unwrap();
        seller
            .escrow
            .deposit()
            .tx_params(tx_params2)
            .call_params(call_params2)
            .call()
            .await
            .unwrap();

        buyer.escrow.approve().call().await.unwrap();
        seller
            .escrow
            .approve()
            .append_variable_outputs(2)
            .call()
            .await
            .unwrap();

        // Should panic
        buyer
            .escrow
            .deposit()
            .tx_params(tx_params3)
            .call_params(call_params3)
            .call()
            .await
            .unwrap();
    }
}

mod approve {

    use super::*;

    #[tokio::test]
    async fn approves() {
        let (deployer, buyer, seller, asset_id, asset_amount) = setup().await;

        let tx_params1 = TxParameters::new(None, Some(1_000_000), None, None);
        let tx_params2 = TxParameters::new(None, Some(1_000_000), None, None);

        let call_params1 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));
        let call_params2 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));

        // Init conditions
        init(
            &deployer,
            &buyer.wallet,
            &seller.wallet,
            asset_id,
            asset_amount,
        )
        .await;
        mint(&deployer, &buyer.wallet, asset_amount).await;
        mint(&deployer, &seller.wallet, asset_amount).await;

        buyer
            .escrow
            .deposit()
            .tx_params(tx_params1)
            .call_params(call_params1)
            .call()
            .await
            .unwrap();
        seller
            .escrow
            .deposit()
            .tx_params(tx_params2)
            .call_params(call_params2)
            .call()
            .await
            .unwrap();

        assert_eq!(
            user_data(&deployer.escrow, &buyer.wallet).await,
            (true, false)
        );
        assert_eq!(
            user_data(&deployer.escrow, &seller.wallet).await,
            (true, false)
        );
        assert_eq!(balance(&deployer.escrow).await, 2 * asset_amount);

        // Test
        assert!(buyer.escrow.approve().call().await.unwrap().value);
        assert!(
            seller
                .escrow
                .approve()
                .append_variable_outputs(2)
                .call()
                .await
                .unwrap()
                .value
        );

        assert_eq!(
            user_data(&deployer.escrow, &buyer.wallet).await,
            (true, true)
        );
        assert_eq!(
            user_data(&deployer.escrow, &seller.wallet).await,
            (true, true)
        );
        assert_eq!(balance(&deployer.escrow).await, 0);
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_when_not_initialized() {
        let (_, buyer, _, _, _) = setup().await;

        // Should panic
        buyer.escrow.approve().call().await.unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_when_sender_is_not_the_correct_address() {
        let (deployer, buyer, seller, asset_id, asset_amount) = setup().await;

        // Init conditions
        init(
            &deployer,
            &buyer.wallet,
            &seller.wallet,
            asset_id,
            asset_amount,
        )
        .await;

        // Should panic
        deployer.escrow.approve().call().await.unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_when_not_deposited() {
        let (deployer, buyer, seller, asset_id, asset_amount) = setup().await;

        // Init conditions
        init(
            &deployer,
            &buyer.wallet,
            &seller.wallet,
            asset_id,
            asset_amount,
        )
        .await;

        // Should panic
        buyer.escrow.approve().call().await.unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_after_both_parties_approve() {
        let (deployer, buyer, seller, asset_id, asset_amount) = setup().await;

        let tx_params1 = TxParameters::new(None, Some(1_000_000), None, None);
        let tx_params2 = TxParameters::new(None, Some(1_000_000), None, None);

        let call_params1 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));
        let call_params2 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));

        // Init conditions
        init(
            &deployer,
            &buyer.wallet,
            &seller.wallet,
            asset_id,
            asset_amount,
        )
        .await;
        mint(&deployer, &buyer.wallet, asset_amount).await;
        mint(&deployer, &seller.wallet, asset_amount).await;

        buyer
            .escrow
            .deposit()
            .tx_params(tx_params1)
            .call_params(call_params1)
            .call()
            .await
            .unwrap();
        seller
            .escrow
            .deposit()
            .tx_params(tx_params2)
            .call_params(call_params2)
            .call()
            .await
            .unwrap();

        buyer.escrow.approve().call().await.unwrap();
        seller
            .escrow
            .approve()
            .append_variable_outputs(2)
            .call()
            .await
            .unwrap();

        // Should panic
        buyer.escrow.approve().call().await.unwrap();
    }
}

mod withdraw {

    use super::*;

    #[tokio::test]
    async fn withdraws() {
        let (deployer, buyer, seller, asset_id, asset_amount) = setup().await;

        let tx_params = TxParameters::new(None, Some(1_000_000), None, None);
        let call_params = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));

        // Init conditions
        init(
            &deployer,
            &buyer.wallet,
            &seller.wallet,
            asset_id,
            asset_amount,
        )
        .await;
        mint(&deployer, &buyer.wallet, asset_amount).await;

        buyer
            .escrow
            .deposit()
            .tx_params(tx_params)
            .call_params(call_params)
            .call()
            .await
            .unwrap();

        buyer.escrow.approve().call().await.unwrap();

        assert_eq!(
            user_data(&deployer.escrow, &buyer.wallet).await,
            (true, true)
        );
        assert_eq!(balance(&deployer.escrow).await, asset_amount);

        // Test
        assert!(
            buyer
                .escrow
                .withdraw()
                .append_variable_outputs(1)
                .call()
                .await
                .unwrap()
                .value
        );

        assert_eq!(
            user_data(&deployer.escrow, &buyer.wallet).await,
            (false, false)
        );
        assert_eq!(balance(&deployer.escrow).await, 0);
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_when_not_initialized() {
        let (_, buyer, _, _, _) = setup().await;

        // Should panic
        buyer.escrow.withdraw().call().await.unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_when_sender_is_not_the_correct_address() {
        let (deployer, buyer, seller, asset_id, asset_amount) = setup().await;

        // Init conditions
        init(
            &deployer,
            &buyer.wallet,
            &seller.wallet,
            asset_id,
            asset_amount,
        )
        .await;

        // Should panic
        deployer.escrow.withdraw().call().await.unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_when_not_deposited() {
        let (deployer, buyer, seller, asset_id, asset_amount) = setup().await;

        // Init conditions
        init(
            &deployer,
            &buyer.wallet,
            &seller.wallet,
            asset_id,
            asset_amount,
        )
        .await;

        // Should panic
        buyer.escrow.withdraw().call().await.unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_after_both_parties_approve() {
        let (deployer, buyer, seller, asset_id, asset_amount) = setup().await;

        let tx_params1 = TxParameters::new(None, Some(1_000_000), None, None);
        let tx_params2 = TxParameters::new(None, Some(1_000_000), None, None);

        let call_params1 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));
        let call_params2 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));

        // Init conditions
        init(
            &deployer,
            &buyer.wallet,
            &seller.wallet,
            asset_id,
            asset_amount,
        )
        .await;
        mint(&deployer, &buyer.wallet, asset_amount).await;
        mint(&deployer, &seller.wallet, asset_amount).await;

        buyer
            .escrow
            .deposit()
            .tx_params(tx_params1)
            .call_params(call_params1)
            .call()
            .await
            .unwrap();
        seller
            .escrow
            .deposit()
            .tx_params(tx_params2)
            .call_params(call_params2)
            .call()
            .await
            .unwrap();

        buyer.escrow.approve().call().await.unwrap();
        seller
            .escrow
            .approve()
            .append_variable_outputs(2)
            .call()
            .await
            .unwrap();

        // Should panic
        buyer
            .escrow
            .withdraw()
            .append_variable_outputs(1)
            .call()
            .await
            .unwrap();
    }
}

mod get_balance {

    use super::*;

    #[tokio::test]
    async fn returns_zero() {
        let (deployer, buyer, seller, asset_id, asset_amount) = setup().await;

        // Init conditions
        init(
            &deployer,
            &buyer.wallet,
            &seller.wallet,
            asset_id,
            asset_amount,
        )
        .await;

        assert_eq!(balance(&deployer.escrow).await, 0);
    }

    #[tokio::test]
    async fn returns_asset_amount() {
        let (deployer, buyer, seller, asset_id, asset_amount) = setup().await;

        let tx_params = TxParameters::new(None, Some(1_000_000), None, None);
        let call_params = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));

        // Init conditions
        init(
            &deployer,
            &buyer.wallet,
            &seller.wallet,
            asset_id,
            asset_amount,
        )
        .await;
        mint(&deployer, &buyer.wallet, asset_amount).await;

        buyer
            .escrow
            .deposit()
            .tx_params(tx_params)
            .call_params(call_params)
            .call()
            .await
            .unwrap();

        assert_eq!(balance(&deployer.escrow).await, asset_amount);
    }
}

mod get_user_data {

    use super::*;

    #[tokio::test]
    async fn gets_user_data() {
        let (deployer, buyer, seller, asset_id, asset_amount) = setup().await;

        let tx_params1 = TxParameters::new(None, Some(1_000_000), None, None);
        let tx_params2 = TxParameters::new(None, Some(1_000_000), None, None);

        let call_params1 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));
        let call_params2 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));

        // Init conditions
        init(
            &deployer,
            &buyer.wallet,
            &seller.wallet,
            asset_id,
            asset_amount,
        )
        .await;
        mint(&deployer, &buyer.wallet, asset_amount).await;
        mint(&deployer, &seller.wallet, asset_amount).await;

        assert_eq!(
            user_data(&deployer.escrow, &buyer.wallet).await,
            (false, false)
        );
        assert_eq!(
            user_data(&deployer.escrow, &seller.wallet).await,
            (false, false)
        );

        buyer
            .escrow
            .deposit()
            .tx_params(tx_params1)
            .call_params(call_params1)
            .call()
            .await
            .unwrap();
        seller
            .escrow
            .deposit()
            .tx_params(tx_params2)
            .call_params(call_params2)
            .call()
            .await
            .unwrap();

        buyer.escrow.approve().call().await.unwrap();
        seller
            .escrow
            .approve()
            .append_variable_outputs(2)
            .call()
            .await
            .unwrap();

        assert_eq!(
            user_data(&deployer.escrow, &buyer.wallet).await,
            (true, true)
        );
        assert_eq!(
            user_data(&deployer.escrow, &seller.wallet).await,
            (true, true)
        );
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_when_not_initialized() {
        let (_, buyer, _, _, _) = setup().await;

        // Should panic
        buyer
            .escrow
            .get_user_data(buyer.wallet.address())
            .call()
            .await
            .unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_when_sender_is_not_the_correct_address() {
        let (deployer, buyer, seller, asset_id, asset_amount) = setup().await;

        // Init conditions
        init(
            &deployer,
            &buyer.wallet,
            &seller.wallet,
            asset_id,
            asset_amount,
        )
        .await;

        // Should panic
        buyer
            .escrow
            .get_user_data(deployer.wallet.address())
            .call()
            .await
            .unwrap();
    }
}

mod get_state {

    use super::*;

    #[tokio::test]
    async fn not_initialized() {
        let (deployer, _, _, _, _) = setup().await;

        assert_eq!(deployer.escrow.get_state().call().await.unwrap().value, 0);
    }

    #[tokio::test]
    async fn initialized() {
        let (deployer, buyer, seller, asset_id, asset_amount) = setup().await;

        // Init conditions
        init(
            &deployer,
            &buyer.wallet,
            &seller.wallet,
            asset_id,
            asset_amount,
        )
        .await;

        assert_eq!(deployer.escrow.get_state().call().await.unwrap().value, 1);
    }

    #[tokio::test]
    async fn completed() {
        let (deployer, buyer, seller, asset_id, asset_amount) = setup().await;

        let tx_params1 = TxParameters::new(None, Some(1_000_000), None, None);
        let tx_params2 = TxParameters::new(None, Some(1_000_000), None, None);

        let call_params1 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));
        let call_params2 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));

        // Init conditions
        assert_eq!(deployer.escrow.get_state().call().await.unwrap().value, 0);

        init(
            &deployer,
            &buyer.wallet,
            &seller.wallet,
            asset_id,
            asset_amount,
        )
        .await;
        mint(&deployer, &buyer.wallet, asset_amount).await;
        mint(&deployer, &seller.wallet, asset_amount).await;

        assert_eq!(deployer.escrow.get_state().call().await.unwrap().value, 1);

        buyer
            .escrow
            .deposit()
            .tx_params(tx_params1)
            .call_params(call_params1)
            .call()
            .await
            .unwrap();
        seller
            .escrow
            .deposit()
            .tx_params(tx_params2)
            .call_params(call_params2)
            .call()
            .await
            .unwrap();

        // Test
        buyer.escrow.approve().call().await.unwrap();
        seller
            .escrow
            .approve()
            .append_variable_outputs(2)
            .call()
            .await
            .unwrap();

        assert_eq!(deployer.escrow.get_state().call().await.unwrap().value, 2);
    }
}
