use anyhow::anyhow;
use fuels::{
    accounts::{wallet::Wallet, Account},
    types::tx_status::TxStatus,
};
use fuels_core::types::{transaction::TxPolicies, Address, AssetId};

#[allow(clippy::too_many_arguments)]
pub async fn transfer(
    wallet: &Wallet,
    recipient: Address,
    amount: u64,
    asset_id: AssetId,
    tx_policies: TxPolicies,
    node: &crate::NodeTarget,
    verbosity: u8,
    writer: &mut impl std::io::Write,
) -> anyhow::Result<super::CallResponse> {
    let provider = wallet.provider();

    // check is recipient is a user
    let tx_response = if provider.is_user_account(*recipient).await? {
        writeln!(
            writer,
            "\nTransferring {} 0x{} to recipient address 0x{}...\n",
            amount, asset_id, recipient
        )?;
        wallet
            .transfer(recipient, amount, asset_id, tx_policies)
            .await
            .map_err(|e| anyhow!("Failed to transfer funds to recipient: {}", e))?
    } else {
        writeln!(
            writer,
            "\nTransferring {} 0x{} to contract address 0x{}...\n",
            amount, asset_id, recipient
        )?;
        let contract_id = (*recipient).into();
        wallet
            .force_transfer_to_contract(contract_id, amount, asset_id, tx_policies)
            .await
            .map_err(|e| anyhow!("Failed to transfer funds to contract: {}", e))?
    };

    super::process_transaction_output(
        TxStatus::Success(tx_response.tx_status),
        &tx_response.tx_id.to_string(),
        &crate::cmd::call::ExecutionMode::Live,
        node,
        verbosity,
        writer,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        op::call::{
            call_function::tests::abigen_bindings::test_contract_mod::std::u128, PrivateKeySigner,
        },
        NodeTarget,
    };
    use fuels::prelude::*;

    #[tokio::test]
    async fn test_transfer_function_to_recipient() {
        // Launch a local network and set up wallets
        let mut wallets = launch_custom_provider_and_get_wallets(
            WalletsConfig::new(
                Some(2),             /* Two wallets */
                Some(1),             /* Single coin (UTXO) */
                Some(1_000_000_000), /* Amount per coin */
            ),
            None,
            None,
        )
        .await
        .unwrap();

        let wallet_sender = wallets.pop().unwrap();
        let wallet_recipient = wallets.pop().unwrap();
        let recipient_address = wallet_recipient.address();

        let provider = wallet_sender.provider();
        let consensus_parameters = provider.consensus_parameters().await.unwrap();
        let base_asset_id = consensus_parameters.base_asset_id();

        // Test helpers to get balances
        let get_recipient_balance = |addr: Address| async move {
            provider
                .get_asset_balance(&addr, base_asset_id)
                .await
                .unwrap()
        };

        // Get initial balance of recipient
        let initial_balance = get_recipient_balance(wallet_recipient.address()).await;

        // Test parameters
        let tx_policies = TxPolicies::default();
        let amount = 100;
        let node = NodeTarget {
            node_url: Some(provider.url().to_string()),
            ..Default::default()
        };

        // should successfully transfer funds
        let response = transfer(
            &wallet_sender,
            recipient_address,
            amount,
            *base_asset_id,
            tx_policies,
            &node,
            0, // verbosity level
            &mut std::io::stdout(),
        )
        .await
        .unwrap();

        // Verify response structure
        assert!(
            !response.tx_hash.is_empty(),
            "Transaction hash should be returned"
        );
        assert!(response.result.is_none(), "Result should be none");

        // Verify balance has increased by the transfer amount
        assert_eq!(
            get_recipient_balance(wallet_recipient.address()).await,
            initial_balance + amount as u128,
            "Balance should increase by transfer amount"
        );
    }

    #[tokio::test]
    async fn test_transfer_function_to_contract() {
        let (_, id, provider, secret_key) = crate::op::call::tests::get_contract_instance().await;

        let wallet = Wallet::new(PrivateKeySigner::new(secret_key), provider.clone());
        let consensus_parameters = provider.clone().consensus_parameters().await.unwrap();
        let base_asset_id = consensus_parameters.base_asset_id();

        // Verify initial contract balance
        let balance = provider
            .get_contract_asset_balance(&id, base_asset_id)
            .await
            .unwrap();
        assert_eq!(balance, 0, "Balance should be 0");

        // Test parameters
        let tx_policies = TxPolicies::default();
        let amount = 100;
        let node = NodeTarget {
            node_url: Some(provider.url().to_string()),
            ..Default::default()
        };

        // should successfully transfer funds
        let response = transfer(
            &wallet,
            Address::new(id.into()),
            amount,
            *base_asset_id,
            tx_policies,
            &node,
            0, // verbosity level
            &mut std::io::stdout(),
        )
        .await
        .unwrap();

        // Verify response structure
        assert!(
            !response.tx_hash.is_empty(),
            "Transaction hash should be returned"
        );
        assert!(response.result.is_none(), "Result should be none");

        // Verify balance has increased by the transfer amount
        let balance = provider
            .get_contract_asset_balance(&id, base_asset_id)
            .await
            .unwrap();
        assert_eq!(
            balance, amount,
            "Balance should increase by transfer amount"
        );
    }
}
