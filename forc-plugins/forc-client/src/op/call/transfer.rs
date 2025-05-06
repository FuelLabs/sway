use anyhow::anyhow;
use fuel_abi_types::abi::program::ProgramABI;
use fuels::{
    accounts::{wallet::Wallet, Account},
    types::bech32::{Bech32Address, Bech32ContractId},
};
use fuels_core::types::{transaction::TxPolicies, Address, AssetId};
use sway_core;

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
            .transfer(&recipient.into(), amount, asset_id, tx_policies)
            .await
            .map_err(|e| anyhow!("Failed to transfer funds to recipient: {}", e))?
    } else {
        writeln!(
            writer,
            "\nTransferring {} 0x{} to contract address 0x{}...\n",
            amount, asset_id, recipient
        )?;
        let address: Bech32Address = recipient.into();
        let contract_id = Bech32ContractId {
            hrp: address.hrp,
            hash: address.hash,
        };
        wallet
            .force_transfer_to_contract(&contract_id, amount, asset_id, tx_policies)
            .await
            .map_err(|e| anyhow!("Failed to transfer funds to contract: {}", e))?
    };

    // We don't need to load the ABI for a simple transfer
    let program_abi = sway_core::asm_generation::ProgramABI::Fuel(ProgramABI::default());
    super::process_transaction_output(
        &tx_response.tx_status.receipts,
        &tx_response.tx_id.to_string(),
        &program_abi,
        None,
        &crate::cmd::call::ExecutionMode::Live,
        node,
        verbosity,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{op::call::PrivateKeySigner, NodeTarget};
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

        let provider = wallet_sender.provider();
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
        assert_eq!(
            response.result.unwrap(),
            "",
            "Result should be empty string"
        );

        // Verify balance has increased by the transfer amount
        assert_eq!(
            get_recipient_balance(wallet_recipient.address().clone()).await,
            initial_balance + amount,
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
            .get_contract_asset_balance(&Bech32ContractId::from(id), *base_asset_id)
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
        assert_eq!(
            response.result.unwrap(),
            "",
            "Result should be empty string"
        );

        // Verify balance has increased by the transfer amount
        let balance = provider
            .get_contract_asset_balance(&Bech32ContractId::from(id), *base_asset_id)
            .await
            .unwrap();
        assert_eq!(
            balance, amount,
            "Balance should increase by transfer amount"
        );
    }
}
