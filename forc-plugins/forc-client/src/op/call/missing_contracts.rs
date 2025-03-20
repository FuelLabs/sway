use anyhow::{bail, Result};
use fuel_tx::{PanicReason, Receipt};
use fuels::programs::calls::{traits::TransactionTuner, ContractCall};
use fuels::types::transaction_builders::BuildableTransaction;
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
pub async fn get_missing_contracts(
    mut call: ContractCall,
    provider: &Provider,
    tx_policies: &TxPolicies,
    variable_output_policy: &VariableOutputPolicy,
    log_decoder: &fuels_core::codec::LogDecoder,
    account: &Wallet<Unlocked<PrivateKeySigner>>,
    max_attempts: Option<u64>,
) -> Result<Vec<Bech32ContractId>> {
    let max_attempts = max_attempts.unwrap_or(10);

    for attempt in 1..=max_attempts {
        forc_tracing::println_warning(&format!(
            "Executing dry-run attempt {} to find missing contracts",
            attempt
        ));

        let tx = call
            .transaction_builder(*tx_policies, *variable_output_policy, account)
            .await?
            .build(provider)
            .await?;

        match provider
            .dry_run(tx)
            .await?
            .take_receipts_checked(Some(log_decoder))
        {
            Ok(_) => return Ok(call.external_contracts),
            Err(fuels_core::types::errors::Error::Transaction(
                fuels::types::errors::transaction::Reason::Reverted { receipts, .. },
            )) => {
                let contract_id = find_id_of_missing_contract(&receipts)?;
                call.external_contracts.push(contract_id);
            }
            Err(err) => bail!(err),
        }
    }
    bail!("Max attempts reached while finding missing contracts")
}

fn find_id_of_missing_contract(receipts: &[Receipt]) -> Result<Bech32ContractId> {
    for receipt in receipts {
        match receipt {
            Receipt::Panic {
                reason,
                contract_id,
                ..
            } if *reason.reason() == PanicReason::ContractNotInInputs => {
                let contract_id = contract_id
                    .expect("panic caused by a contract not in inputs must have a contract id");
                return Ok(Bech32ContractId::from(contract_id));
            }
            Receipt::Panic { reason, .. } => {
                // If it's a panic but not ContractNotInInputs, include the reason
                bail!("Contract execution panicked with reason: {:?}", reason);
            }
            _ => continue,
        }
    }
    bail!("No contract found in receipts: {:?}", receipts)
}
