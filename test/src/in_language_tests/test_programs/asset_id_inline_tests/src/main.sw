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

#[test]
fn asset_id_try_from_bytes() {
    use std::bytes::Bytes;

    // Test empty bytes
    let bytes_1 = Bytes::new();
    assert(AssetId::try_from(bytes_1).is_none());

    // Test not full length but capacity bytes
    let mut bytes_2 = Bytes::with_capacity(32);
    bytes_2.push(1u8);
    bytes_2.push(3u8);
    bytes_2.push(5u8);
    assert(AssetId::try_from(bytes_2).is_none());

    // Test zero bytes
    let bytes_3 = Bytes::from(b256::zero());
    let asset_id_3 = AssetId::try_from(bytes_3);
    assert(asset_id_3.is_some());
    assert(asset_id_3.unwrap() == AssetId::zero());

    // Test max bytes
    let bytes_4 = Bytes::from(b256::max());
    let asset_id_4 = AssetId::try_from(bytes_4);
    assert(asset_id_4.is_some());
    assert(asset_id_4.unwrap() == AssetId::from(b256::max()));

    // Test too many bytes
    let mut bytes_5 = Bytes::from(b256::max());
    bytes_5.push(255u8);
    assert(AssetId::try_from(bytes_5).is_none());

    // Test modifying bytes after doesn't impact 
    let mut bytes_6 = Bytes::from(b256::zero());
    let asset_id_6 = AssetId::try_from(bytes_6);
    assert(asset_id_6.is_some());
    assert(asset_id_6.unwrap() == AssetId::zero());
    bytes_6.set(0, 255u8);
    assert(asset_id_6.unwrap() == AssetId::zero());
}

#[test]
fn asset_id_try_into_bytes() {
    use std::bytes::Bytes;

    let asset_id_1 = AssetId::zero();
    let bytes_1: Bytes = <AssetId as Into<Bytes>>::into(asset_id_1);
    assert(bytes_1.capacity() == 32);
    assert(bytes_1.len() == 32);
    let mut iter_1 = 0;
    while iter_1 < 32 {
        assert(bytes_1.get(iter_1).unwrap() == 0u8);
        iter_1 += 1;
    }

    let asset_id_2 = AssetId::from(b256::max());
    let bytes_2: Bytes = <AssetId as Into<Bytes>>::into(asset_id_2);
    assert(bytes_2.capacity() == 32);
    assert(bytes_2.len() == 32);
    let mut iter_2 = 0;
    while iter_2 < 32 {
        assert(bytes_2.get(iter_2).unwrap() == 255u8);
        iter_2 += 1;
    }

    let asset_id_3 = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let bytes_3: Bytes = <AssetId as Into<Bytes>>::into(asset_id_3);
    assert(bytes_3.capacity() == 32);
    assert(bytes_3.len() == 32);
    assert(bytes_3.get(31).unwrap() == 1u8);
    let mut iter_3 = 0;
    while iter_3 < 31 {
        assert(bytes_3.get(iter_3).unwrap() == 0u8);
        iter_3 += 1;
    }
}

#[test]
fn asset_id_fuel() {
    let fuel_asset = AssetId::fuel();
    // Testnet asset id
    assert(fuel_asset.is_ok());
    assert(
        fuel_asset
            .unwrap()
            .bits() == 0x324d0c35a4299ef88138a656d5272c5a3a9ccde2630ae055dacaf9d13443d53b,
    );
}

#[test]
fn asset_id_usdc() {
    let usdc_asset = AssetId::usdc();
    // Testnet asset id 
    assert(usdc_asset.is_ok());
    assert(
        usdc_asset
            .unwrap()
            .bits() == 0xc26c91055de37528492e7e97d91c6f4abe34aae26f2c4d25cff6bfe45b5dc9a9,
    );
}

#[test]
fn asset_id_usde() {
    let usde_asset = AssetId::usde();
    // Testnet asset id 
    assert(usde_asset.is_ok());
    assert(
        usde_asset
            .unwrap()
            .bits() == 0x86a1beb50c844f5eff9afd21af514a13327c93f76edb89333af862f70040b107,
    );
}

#[test]
fn asset_id_susde() {
    let susde_asset = AssetId::susde();
    // Testnet asset id 
    assert(susde_asset.is_ok());
    assert(
        susde_asset
            .unwrap()
            .bits() == 0xd2886b34454e2e0de47a82d8e6314b26e1e1312519247e8e2ef137672a909aeb,
    );
}

#[test]
fn asset_id_wsteth() {
    let wsteth_asset = AssetId::wsteth();
    // Testnet asset id 
    assert(wsteth_asset.is_ok());
    assert(
        wsteth_asset
            .unwrap()
            .bits() == 0xb42cd9ddf61898da1701adb3a003b0cf4ca6df7b5fe490ec2c295b1ca43b33c8,
    );
}

#[test]
fn asset_id_weth() {
    let weth_asset = AssetId::weth();
    // No verified testnet asset id
    assert(weth_asset.is_err());
}

#[test]
fn asset_id_usdt() {
    let usdt_asset = AssetId::usdt();
    // No verified testnet asset id
    assert(usdt_asset.is_err());
}

#[test]
fn asset_id_weeth() {
    let weeth_asset = AssetId::weeth();
    // No verified testnet asset id
    assert(weeth_asset.is_err());
}

#[test]
fn asset_id_rseth() {
    let rseth_asset = AssetId::rseth();
    // No verified testnet asset id
    assert(rseth_asset.is_err());
}

#[test]
fn asset_id_reth() {
    let reth_asset = AssetId::reth();
    // No verified testnet asset id
    assert(reth_asset.is_err());
}

#[test]
fn asset_id_wbeth() {
    let wbeth_asset = AssetId::wbeth();
    // No verified testnet asset id
    assert(wbeth_asset.is_err());
}

#[test]
fn asset_id_rsteth() {
    let rsteth_asset = AssetId::rsteth();
    // No verified testnet asset id
    assert(rsteth_asset.is_err());
}

#[test]
fn asset_id_amphreth() {
    let amphreth_asset = AssetId::amphreth();
    // No verified testnet asset id
    assert(amphreth_asset.is_err());
}

#[test]
fn asset_id_manta_mbtc() {
    let manta_mbtc_asset = AssetId::manta_mbtc();
    // No verified testnet asset id
    assert(manta_mbtc_asset.is_err());
}

#[test]
fn asset_id_manta_meth() {
    let manta_meth_asset = AssetId::manta_meth();
    // No verified testnet asset id
    assert(manta_meth_asset.is_err());
}

#[test]
fn asset_id_manta_musd() {
    let manta_musd_asset = AssetId::manta_musd();
    // No verified testnet asset id
    assert(manta_musd_asset.is_err());
}

#[test]
fn asset_id_pumpbtc() {
    let pumpbtc_asset = AssetId::pumpbtc();
    // No verified testnet asset id
    assert(pumpbtc_asset.is_err());
}

#[test]
fn asset_id_fbtc() {
    let fbtc_asset = AssetId::fbtc();
    // No verified testnet asset id
    assert(fbtc_asset.is_err());
}

#[test]
fn asset_id_solvbtc() {
    let solvbtc_asset = AssetId::solvbtc();
    // No verified testnet asset id
    assert(solvbtc_asset.is_err());
}

#[test]
fn asset_id_solvbtc_bnn() {
    let solvbtc_bnn_asset = AssetId::solvbtc_bnn();
    // No verified testnet asset id
    assert(solvbtc_bnn_asset.is_err());
}

#[test]
fn asset_id_mantle_meth() {
    let mantle_meth_asset = AssetId::mantle_meth();
    // No verified testnet asset id
    assert(mantle_meth_asset.is_err());
}

#[test]
fn asset_id_sdai() {
    let sdai_asset = AssetId::sdai();
    // No verified testnet asset id
    assert(sdai_asset.is_err());
}

#[test]
fn asset_id_rsusde() {
    let rsusde_asset = AssetId::rsusde();
    // No verified testnet asset id
    assert(rsusde_asset.is_err());
}

#[test]
fn asset_id_ezeth() {
    let ezeth_asset = AssetId::ezeth();
    // No verified testnet asset id
    assert(ezeth_asset.is_err());
}

#[test]
fn asset_id_pzeth() {
    let pzeth_asset = AssetId::pzeth();
    // No verified testnet asset id
    assert(pzeth_asset.is_err());
}

#[test]
fn asset_id_re7lrt() {
    let re7lrt_asset = AssetId::re7lrt();
    // No verified testnet asset id
    assert(re7lrt_asset.is_err());
}

#[test]
fn asset_id_steaklrt() {
    let steaklrt_asset = AssetId::steaklrt();
    // No verified testnet asset id
    assert(steaklrt_asset.is_err());
}

#[test]
fn asset_id_usdf() {
    let usdf_asset = AssetId::usdf();
    // No verified testnet asset id
    assert(usdf_asset.is_err());
}
