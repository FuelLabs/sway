predicate;

use std::outputs::{Output, output_asset_id, output_asset_to, output_asset_id_and_to, output_type};

fn main(index: u64, asset_id: b256, to: b256, expected_type: Output) -> bool {
    let tx_asset_id = output_asset_id(index);
    let tx_to = output_asset_to(index);
    let tx_output_type = output_type(index);
    let tx_asset_id_and_to = output_asset_id_and_to(index);

    assert(tx_asset_id.is_some() && tx_asset_id.unwrap().bits() == asset_id);
    assert(tx_to.is_some() && tx_to.unwrap().bits() == to);
    assert(tx_output_type.is_some() && tx_output_type.unwrap() == expected_type);
    assert(tx_asset_id_and_to.is_some() && tx_asset_id.unwrap() == tx_asset_id_and_to.unwrap().0);
    assert(tx_to.unwrap() == tx_asset_id_and_to.unwrap().1);

    true
}
