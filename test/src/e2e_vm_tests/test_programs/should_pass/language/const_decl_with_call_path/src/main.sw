script;

mod test_lib;

fn main() -> u64 {
    let x = test_lib::NUMBER;
    let zero = b256::zero();
    let base_asset_id = AssetId::base();
    let base_asset_id_b256: b256 = base_asset_id.into();
    assert(zero == 0x0000000000000000000000000000000000000000000000000000000000000000);
    assert(base_asset_id_b256 == zero);
    x
}
