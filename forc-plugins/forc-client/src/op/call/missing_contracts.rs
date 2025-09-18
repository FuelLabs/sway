use anyhow::{bail, Result};
use fuel_tx::ContractId;
use fuels::programs::calls::{
    traits::TransactionTuner, utils::find_ids_of_missing_contracts, ContractCall,
};
use fuels_accounts::{
    provider::Provider,
    signers::private_key::PrivateKeySigner,
    wallet::{Unlocked, Wallet},
    ViewOnlyAccount,
};
use fuels_core::types::{transaction::TxPolicies, transaction_builders::VariableOutputPolicy};

/// Get the missing contracts from a contract call by dry-running the transaction
/// to find contracts that are not explicitly listed in the call's `external_contracts` field.
/// Note: This function is derived from `determine_missing_contracts` in `fuels-rs`
pub async fn determine_missing_contracts(
    call: &ContractCall,
    provider: &Provider,
    tx_policies: &TxPolicies,
    variable_output_policy: &VariableOutputPolicy,
    log_decoder: &fuels_core::codec::LogDecoder,
    account: &Wallet<Unlocked<PrivateKeySigner>>,
) -> Result<Vec<ContractId>> {
    let consensus_parameters = account.provider().consensus_parameters().await?;

    let required_asset_amounts = call.required_assets(*consensus_parameters.base_asset_id());

    // Find the spendable resources required for those calls
    let mut asset_inputs = vec![];
    for &(asset_id, amount) in &required_asset_amounts {
        let resources = account
            .get_asset_inputs_for_amount(asset_id, amount, None)
            .await?;
        asset_inputs.extend(resources);
    }

    let tb = call.transaction_builder(
        *tx_policies,
        *variable_output_policy,
        &consensus_parameters,
        asset_inputs,
        account,
    )?;

    let tx = call
        .build_tx(tb, account)
        .await
        .expect("Failed to build transaction");

    match provider
        .dry_run(tx)
        .await?
        .take_receipts_checked(Some(log_decoder))
    {
        Ok(_) => Ok(vec![]),
        Err(fuels_core::types::errors::Error::Transaction(
            fuels::types::errors::transaction::Reason::Failure { receipts, .. },
        )) => {
            let missing_contracts = find_ids_of_missing_contracts(&receipts);
            Ok(missing_contracts)
        }
        Err(err) => bail!(err),
    }
}
