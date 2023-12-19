use anyhow::Result;
use forc_tx::Gas;
use fuel_core_client::client::types::NodeInfo;
use fuel_tx::{
    field::{Inputs, Witnesses},
    Buildable, Chargeable, Input, Script, TxPointer,
};
use fuels_accounts::provider::Provider;
use fuels_core::types::transaction_builders::DryRunner;

/// Returns the gas to use for deployment, overriding default values if necessary.
pub fn get_gas_price(gas: &Gas, node_info: NodeInfo) -> u64 {
    // TODO: write unit tests for this function once https://github.com/FuelLabs/fuel-core/issues/1312 is resolved.
    if let Some(gas_price) = gas.price {
        gas_price
    } else {
        node_info.min_gas_price
    }
}

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

pub(crate) async fn get_gas_used(mut tx: Script, provider: &Provider) -> Result<u64> {
    let no_spendable_input = no_spendable_input(tx.inputs());
    if no_spendable_input {
        tx.inputs_mut().push(Input::coin_signed(
            Default::default(),
            Default::default(),
            1_000_000_000,
            Default::default(),
            TxPointer::default(),
            0,
            0u32.into(),
        ));

        // Add an empty `Witness` for the `coin_signed` we just added
        // and increase the witness limit
        tx.witnesses_mut().push(Default::default());
    }

    // Get `max_gas` used by everything except the script execution. Add `1` because of rounding.
    let network_info = provider.network_info().await?;
    let consensus_params = &network_info.consensus_parameters;
    let max_gas = tx.max_gas(consensus_params.gas_costs(), consensus_params.fee_params()) + 1;
    // Increase `script_gas_limit` to the maximum allowed value.
    tx.set_script_gas_limit(network_info.max_gas_per_tx() - max_gas);

    let tolerance = 0.1;
    let gas_used = provider
        .dry_run_and_get_used_gas(tx.clone().into(), tolerance)
        .await?;

    Ok(gas_used)
}
