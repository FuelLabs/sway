predicate;

use std::outputs::{output_asset_id, output_asset_to};

fn main(index: u64, asset_id: b256, to: b256) -> bool {
    let tx_asset_id = output_asset_id(index);
    let tx_to = output_asset_to(index);

    assert(tx_asset_id.is_some() && tx_asset_id.unwrap().bits() == asset_id);
    assert(tx_to.is_some() && tx_to.unwrap().bits() == to);

    true
}
