use anyhow::Result;
use fuel_tx::{
    field::{Inputs, Witnesses},
    Buildable, Chargeable, Input, Script, Transaction, TxPointer,
};
use fuels_accounts::provider::Provider;
use fuels_core::types::transaction_builders::DryRunner;

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
    if no_spendable_input {
        tx.inputs_mut().push(Input::coin_signed(
            Default::default(),
            Default::default(),
            1_000_000_000,
            Default::default(),
            TxPointer::default(),
            0,
        ));

        // Add an empty `Witness` for the `coin_signed` we just added
        // and increase the witness limit
        tx.witnesses_mut().push(Default::default());
    }

    // Get `max_gas` used by everything except the script execution. Add `1` because of rounding.
    let consensus_params = provider.consensus_parameters();
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
    println!("estimation {gas_used}");
    Ok(gas_used)
}
