script;

fn main() -> bool {
    use std::bytes::Bytes;
    
    let my_asset = AssetId::zero();
    let my_bytes: Bytes = my_asset.into();

    true
}
