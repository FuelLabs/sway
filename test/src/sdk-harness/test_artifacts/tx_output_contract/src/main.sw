contract;

use std::asset::transfer;
use std::outputs::*;

abi TxOutputContract {
    fn send_assets_change(to: Address, asset: AssetId, amount: u64);
    fn send_assets_variable(to: Address, asset: AssetId, index: u64) -> (Address, AssetId, u64);
}

impl TxOutputContract for Contract {
    fn send_assets_change(to: Address, asset: AssetId, amount: u64) {
        transfer(Identity::Address(to), asset, amount);
    }

    fn send_assets_variable(to: Address, asset: AssetId, index: u64) -> (Address, AssetId, u64) {
        transfer(Identity::Address(to), asset, 1);

        get_variable_tx_params(index)
    }
}

fn get_variable_tx_params(index: u64) -> (Address, AssetId, u64) {
    let tx_asset_id = output_asset_id(index);
    let tx_to = output_asset_to(index);
    let tx_amount = output_amount(index);
    let tx_asset_id_and_to = output_asset_id_and_to(index);

    let tx_output_type = output_type(index);
    assert(tx_output_type.is_some() && tx_output_type.unwrap() == Output::Variable);
    assert(
        (tx_asset_id.is_some() && tx_asset_id_and_to.is_some() && tx_asset_id.unwrap() == tx_asset_id_and_to.unwrap().0)
        ||
        (tx_asset_id.is_none() && tx_asset_id_and_to.is_none())
    );
    assert(
        (tx_to.is_some() && tx_asset_id_and_to.is_some() && tx_to.unwrap() == tx_asset_id_and_to.unwrap().1)
        ||
        (tx_to.is_none() && tx_asset_id_and_to.is_none())
    );

    (
        tx_to.unwrap_or(Address::zero()),
        tx_asset_id.unwrap_or(AssetId::zero()),
        tx_amount.unwrap_or(0),
    )
}
