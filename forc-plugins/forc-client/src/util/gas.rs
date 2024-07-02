use anyhow::Result;

use fuel_tx::{
    field::{Inputs, Witnesses},
    Buildable, Chargeable, Input, Script, TxPointer,
};
use fuels_accounts::provider::Provider;
use fuels_core::types::transaction::ScriptTransaction;

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
    let script_tx = ScriptTransaction::from(tx);

    let tolerance = 0.1;
    let estimated_tx_cost = provider
        .estimate_transaction_cost(script_tx, Some(tolerance), None)
        .await?;
    Ok(estimated_tx_cost.gas_used)
}
