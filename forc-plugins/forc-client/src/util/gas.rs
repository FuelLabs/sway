use forc_tx::Gas;
use fuel_core_client::client::types::NodeInfo;

/// Returns the gas to use for deployment, overriding default values if necessary.
pub fn get_gas_price(gas: &Gas, node_info: NodeInfo) -> u64 {
    // TODO: write unit tests for this function once https://github.com/FuelLabs/fuel-core/issues/1312 is resolved.
    if let Some(gas_price) = gas.price {
        gas_price
    } else {
        node_info.min_gas_price
    }
}
