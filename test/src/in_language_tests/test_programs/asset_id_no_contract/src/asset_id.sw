library;

#[test]
fn asset_id_default_not_in_contract() {
    // Because this is not within a contract context, this will return erroneous data
    let _asset = AssetId::default();
}
