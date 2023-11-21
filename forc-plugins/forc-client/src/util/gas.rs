use forc_tx::Gas;
use fuel_core_client::client::types::{ChainInfo, NodeInfo};

/// Returns the gas to use for deployment, overriding default values if necessary.
pub fn get_gas_price(gas: &Gas, node_info: NodeInfo) -> u64 {
    // TODO: write unit tests for this function once https://github.com/FuelLabs/fuel-core/issues/1312 is resolved.
    if let Some(gas_price) = gas.price {
        gas_price
    } else {
        node_info.min_gas_price
    }
}

/// Returns the gas to use for deployment, overriding default values if necessary.
pub fn get_gas_limit(gas: &Gas, chain_info: ChainInfo) -> u64 {
    // TODO: write unit tests for this function once https://github.com/FuelLabs/fuel-core/issues/1312 is resolved.
    if let Some(gas_limit) = gas.limit {
        gas_limit
    } else {
        //TODO: @hal3e estimate real `gas_used`
        chain_info.consensus_parameters.tx_params().max_gas_per_tx / 2
    }
}
