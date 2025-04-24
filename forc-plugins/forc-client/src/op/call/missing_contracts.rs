use anyhow::{bail, Result};
use fuels::programs::calls::{
    traits::TransactionTuner, utils::find_ids_of_missing_contracts, ContractCall,
};
use fuels_accounts::{
    provider::Provider,
    signers::private_key::PrivateKeySigner,
    wallet::{Unlocked, Wallet},
};
use fuels_core::types::{
    bech32::Bech32ContractId, transaction::TxPolicies, transaction_builders::VariableOutputPolicy,
};

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
) -> Result<Vec<Bech32ContractId>> {
    let tb = call
        .transaction_builder(*tx_policies, *variable_output_policy, account)
        .await
        .expect("Failed to initialize transaction builder");

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
