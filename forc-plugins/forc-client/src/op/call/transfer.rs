use anyhow::anyhow;
use fuel_abi_types::abi::program::ProgramABI;
use fuels::accounts::{wallet::WalletUnlocked, Account};
use fuels_core::types::{transaction::TxPolicies, Address, AssetId};
use sway_core;

pub async fn transfer(
    wallet: &WalletUnlocked,
    recipient: Address,
    amount: u64,
    asset_id: AssetId,
    tx_policies: TxPolicies,
    show_receipts: bool,
    node: &crate::NodeTarget,
) -> anyhow::Result<super::CallResponse> {
    println!(
        "\nTransferring {} 0x{} to address 0x{}...\n",
        amount, asset_id, recipient
    );

    let (tx_hash, receipts) = wallet
        .transfer(&recipient.into(), amount, asset_id, tx_policies)
        .await
        .map_err(|e| anyhow!("Failed to transfer funds: {}", e))?;

    // We don't need to load the ABI for a simple transfer
    let program_abi = sway_core::asm_generation::ProgramABI::Fuel(ProgramABI::default());
    super::process_transaction_output(
        &receipts,
        &tx_hash.to_string(),
        &program_abi,
        "".to_string(),
        &crate::cmd::call::ExecutionMode::Live,
        node,
        show_receipts,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeTarget;
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
        let recipient_address = wallet_recipient.address().into();

        let provider = wallet_sender.provider().unwrap();
        let consensus_parameters = provider.consensus_parameters().await.unwrap();
        let base_asset_id = consensus_parameters.base_asset_id();

        // Test helpers to get balances
        let get_recipient_balance = |addr: Bech32Address| async move {
            provider
                .get_asset_balance(&addr, *base_asset_id)
                .await
                .unwrap()
        };

        // Get initial balance of recipient
        let initial_balance = get_recipient_balance(wallet_recipient.address().clone()).await;

        // Test parameters
        let tx_policies = TxPolicies::default();
        let amount = 100;
        let node = NodeTarget {
            node_url: Some(provider.url().to_string()),
            ..Default::default()
        };

        // should successfully transfer funds)
        let result = transfer(
            &wallet_sender,
            recipient_address,
            amount,
            *base_asset_id,
            tx_policies,
            false, // show_receipts
            &node,
        )
        .await
        .unwrap();

        // Verify response structure
        assert!(
            !result.tx_hash.is_empty(),
            "Transaction hash should be returned"
        );
        assert_eq!(result.result, "", "Result should be empty string");

        // Verify balance has increased by the transfer amount
        assert_eq!(
            get_recipient_balance(wallet_recipient.address().clone()).await,
            initial_balance + amount,
            "Balance should increase by transfer amount"
        );
    }
}
