library;

#[test]
fn asset_id_hasher() {
    use std::hash::{Hash, sha256};

    let asset_1 = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let digest_1 = sha256(asset_1);
    assert(digest_1 == 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);

    let asset_2 = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let digest_2 = sha256(asset_2);
    assert(digest_2 == 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);

    let asset_3 = AssetId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let digest3 = sha256(asset_3);
    assert(digest3 == 0xaf9613760f72635fbdb44a5a0a63c39f12af30f950a6ee5c971be188e89c4051);
}

#[test]
fn asset_id_eq() {
    let asset_1 = AssetId::zero();
    let asset_2 = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let asset_3 = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let asset_4 = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let asset_5 = AssetId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let asset_6 = AssetId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);

    assert(asset_1 == asset_2);
    assert(asset_3 == asset_4);
    assert(asset_5 == asset_6);
}

#[test]
fn asset_id_ne() {
    let asset_1 = AssetId::zero();
    let asset_2 = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let asset_3 = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let asset_4 = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let asset_5 = AssetId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let asset_6 = AssetId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);

    assert(asset_1 != asset_3);
    assert(asset_1 != asset_4);
    assert(asset_1 != asset_5);
    assert(asset_1 != asset_6);
    assert(asset_2 != asset_3);
    assert(asset_2 != asset_4);
    assert(asset_2 != asset_5);
    assert(asset_2 != asset_6);
    assert(asset_3 != asset_5);
    assert(asset_3 != asset_6);
}

#[test]
fn asset_id_from_b256() {
    let asset1 = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    assert(
        asset1
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let asset2 = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    assert(
        asset2
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );

    let asset3 = AssetId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    assert(
        asset3
            .bits() == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );
}

#[test]
fn asset_id_b256_into() {
    let b256_1 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let asset_1: AssetId = b256_1.into();
    assert(
        asset_1
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let b256_2 = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let asset_2: AssetId = b256_2.into();
    assert(
        asset_2
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );

    let b256_3 = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
    let asset_3: AssetId = b256_3.into();
    assert(
        asset_3
            .bits() == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );
}

#[test]
fn asset_id_into_b256() {
    let asset_1 = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let b256_1: b256 = asset_1.into();
    assert(b256_1 == 0x0000000000000000000000000000000000000000000000000000000000000000);

    let asset_2 = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let b256_2: b256 = asset_2.into();
    assert(b256_2 == 0x0000000000000000000000000000000000000000000000000000000000000001);

    let asset_3 = AssetId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let b256_3: b256 = asset_3.into();
    assert(b256_3 == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
}

#[test]
fn asset_id_b256_from() {
    let asset_1 = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let b256_1: b256 = b256::from(asset_1);
    assert(b256_1 == 0x0000000000000000000000000000000000000000000000000000000000000000);

    let asset_2 = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let b256_2: b256 = b256::from(asset_2);
    assert(b256_2 == 0x0000000000000000000000000000000000000000000000000000000000000001);

    let asset_3 = AssetId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let b256_3: b256 = b256::from(asset_3);
    assert(b256_3 == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
}

#[test]
fn asset_id_new() {
    let contract_id_1 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let sub_id_1 = SubId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let asset_1 = AssetId::new(contract_id_1, sub_id_1);
    assert(
        asset_1
            .bits() == 0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b,
    );

    let contract_id_2 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let sub_id_2 = SubId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let asset_2 = AssetId::new(contract_id_2, sub_id_2);
    assert(
        asset_2
            .bits() == 0x58e8f2a1f78f0a591feb75aebecaaa81076e4290894b1c445cc32953604db089,
    );

    let contract_id_3 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let sub_id_3 = SubId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let asset_3 = AssetId::new(contract_id_3, sub_id_3);
    assert(
        asset_3
            .bits() == 0x90f4b39548df55ad6187a1d20d731ecee78c545b94afd16f42ef7592d99cd365,
    );

    let contract_id_4 = ContractId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let sub_id_4 = SubId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let asset_4 = AssetId::new(contract_id_4, sub_id_4);
    assert(
        asset_4
            .bits() == 0xa5de9b714accd8afaaabf1cbd6e1014c9d07ff95c2ae154d91ec68485b31e7b5,
    );

    let contract_id_5 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let sub_id_5 = SubId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let asset_5 = AssetId::new(contract_id_5, sub_id_5);
    assert(
        asset_5
            .bits() == 0xbba91ca85dc914b2ec3efb9e16e7267bf9193b14350d20fba8a8b406730ae30a,
    );

    let contract_id_6 = ContractId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let sub_id_6 = SubId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let asset_6 = AssetId::new(contract_id_6, sub_id_6);
    assert(
        asset_6
            .bits() == 0x8667e718294e9e0df1d30600ba3eeb201f764aad2dad72748643e4a285e1d1f7,
    );

    let contract_id_7 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let sub_id_7 = SubId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let asset_7 = AssetId::new(contract_id_7, sub_id_7);
    assert(
        asset_7
            .bits() == 0x4ab9077c34a6903bc59693414a4fe8ccf275d93f2daacd849574933737c27757,
    );

    let contract_id_8 = ContractId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let sub_id_8 = SubId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let asset_8 = AssetId::new(contract_id_8, sub_id_8);
    assert(
        asset_8
            .bits() == 0x2a9bb11102924faefcdbd39baa7858c5f5e49ed2a4205f6759c4a8648bee2942,
    );
}

#[test]
fn asset_id_default_not_in_contract() {
    // Because this is not within a contract context, this will return erroneous data
    let _asset = AssetId::default();
}

#[test]
fn asset_id_base() {
    let base_asset = AssetId::base();
    assert(
        base_asset
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000000,
    );
}

#[test]
fn asset_id_bits() {
    let asset1 = AssetId::zero();
    assert(
        asset1
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let asset2 = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    assert(
        asset2
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );

    let asset3 = AssetId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    assert(
        asset3
            .bits() == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );
}

#[test]
fn asset_id_zero() {
    let zero_asset = AssetId::zero();
    assert(
        zero_asset
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000000,
    );
}

#[test]
fn asset_id_is_zero() {
    let zero_asset = AssetId::zero();
    assert(zero_asset.is_zero());

    let asset_2 = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    assert(!asset_2.is_zero());

    let asset_3 = AssetId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    assert(!asset_3.is_zero());
}
