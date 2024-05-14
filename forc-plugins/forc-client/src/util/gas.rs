use anyhow::Result;

use fuel_core_client::client::FuelClient;
use fuel_core_types::services::executor::TransactionExecutionResult;
use fuel_tx::{
    field::{Inputs, MaxFeeLimit, Witnesses},
    Buildable, Chargeable, Create, Input, Script, Transaction, TxPointer,
};
use fuels_accounts::provider::Provider;
use fuels_core::{
    constants::DEFAULT_GAS_ESTIMATION_BLOCK_HORIZON, types::transaction_builders::DryRunner,
};

fn no_spendable_input<'a, I: IntoIterator<Item = &'a Input>>(inputs: I) -> bool {
    !inputs.into_iter().any(|i| {
        matches!(
            i,
            Input::CoinSigned(_)
                | Input::CoinPredicate(_)
                | Input::MessageCoinSigned(_)
                | Input::MessageCoinPredicate(_)
        )
    })
}

pub(crate) async fn get_script_gas_used(mut tx: Script, provider: &Provider) -> Result<u64> {
    let no_spendable_input = no_spendable_input(tx.inputs());
    let base_asset_id = provider.base_asset_id();
    if no_spendable_input {
        tx.inputs_mut().push(Input::coin_signed(
            Default::default(),
            Default::default(),
            1_000_000_000,
            *base_asset_id,
            TxPointer::default(),
            0,
        ));

        // Add an empty `Witness` for the `coin_signed` we just added
        // and increase the witness limit
        tx.witnesses_mut().push(Default::default());
    }
    let consensus_params = provider.consensus_parameters();

    // Get `max_gas` used by everything except the script execution. Add `1` because of rounding.
    let max_gas_per_tx = consensus_params.tx_params().max_gas_per_tx();
    let max_gas = tx.max_gas(consensus_params.gas_costs(), consensus_params.fee_params()) + 1;
    // Increase `script_gas_limit` to the maximum allowed value.
    tx.set_script_gas_limit(max_gas_per_tx - max_gas);

    get_gas_used(Transaction::Script(tx), provider).await
}

/// Returns gas_used for an arbitrary tx, by doing dry run with the provided `Provider`.
pub(crate) async fn get_gas_used(tx: Transaction, provider: &Provider) -> Result<u64> {
    let tolerance = 0.1;
    let gas_used = provider.dry_run_and_get_used_gas(tx, tolerance).await?;
    Ok(gas_used)
}

/// Returns an estimation for the max fee of `Create` transactions.
/// Accepts a `tolerance` which is used to add some safety margin to the estimation.
/// Resulting estimation is calculated as `(dry_run_estimation * tolerance)/100 + dry_run_estimation)`.
pub(crate) async fn get_estimated_max_fee(
    tx: Create,
    provider: &Provider,
    client: &FuelClient,
    tolerance: u64,
) -> Result<u64> {
    let mut tx = tx.clone();
    // Add dummy input to get past validation for dry run.
    let no_spendable_input = no_spendable_input(tx.inputs());
    let base_asset_id = provider.base_asset_id();
    if no_spendable_input {
        tx.inputs_mut().push(Input::coin_signed(
            Default::default(),
            Default::default(),
            1_000_000_000,
            *base_asset_id,
            TxPointer::default(),
            0,
        ));

        // Add an empty `Witness` for the `coin_signed` we just added
        // and increase the witness limit
        tx.witnesses_mut().push(Default::default());
    }
    let consensus_params = provider.consensus_parameters();
    let gas_price = provider
        .estimate_gas_price(DEFAULT_GAS_ESTIMATION_BLOCK_HORIZON)
        .await?
        .gas_price;
    let max_fee = tx.max_fee(
        consensus_params.gas_costs(),
        consensus_params.fee_params(),
        gas_price,
    );
    tx.set_max_fee_limit(max_fee as u64);
    let tx = Transaction::from(tx);

    let tx_status = client
        .dry_run_opt(&[tx], Some(false))
        .await
        .map(|mut status_vec| status_vec.remove(0))?;
    let total_fee = match tx_status.result {
        TransactionExecutionResult::Success { total_fee, .. } => total_fee,
        TransactionExecutionResult::Failed { total_fee, .. } => total_fee,
    };

    let total_fee_with_tolerance = ((total_fee * tolerance) / 100) + total_fee;
    Ok(total_fee_with_tolerance)
}
