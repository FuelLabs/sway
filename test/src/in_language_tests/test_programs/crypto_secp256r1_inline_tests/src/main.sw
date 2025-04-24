library;

use std::{
    b512::B512,
    bytes::Bytes,
    crypto::{
        message::*,
        public_key::*,
        secp256r1::*,
    },
    hash::{
        Hash,
        sha256,
    },
    vm::evm::evm_address::EvmAddress,
};

#[test]
fn secp256r1_new() {
    let new_secp256r1 = Secp256r1::new();
    let mut iter = 0;
    while iter < 64 {
        assert(new_secp256r1.bits()[iter] == 0u8);
        iter += 1;
    }
}

#[test]
fn secp256r1_bits() {
    let new_secp256r1 = Secp256r1::new();
    let secp256r1_bits = new_secp256r1.bits();
    let mut iter = 0;
    while iter < 64 {
        assert(secp256r1_bits[iter] == 0u8);
        iter += 1;
    }
}

#[test]
fn secp256r1_recover() {
    let hi = 0xbd0c9b8792876712afadbff382e1bf31c44437823ed761cc3600d0016de511ac;
    let lo = 0x44ac566bd156b4fc71a4a4cb2655d3da360c695edb27dc3b64d621e122fea23d;
    let msg_hash = 0x1e45523606c96c98ba970ff7cf9511fab8b25e1bcd52ced30b81df1e4a9c4323;
    let pub_hi = 0xd6ea577a54ae42411fbc78d686d4abba2150ca83540528e4b868002e346004b2;
    let pub_lo = 0x62660ecce5979493fe5684526e8e00875b948e507a89a47096bc84064a175452;
    let signature: Secp256r1 = Secp256r1::from((hi, lo));
    let public_key = PublicKey::from((pub_hi, pub_lo));
    let message = Message::from(msg_hash);

    // A recovered public key pair.
    let result_public_key = signature.recover(message);
    assert(result_public_key.is_ok());
    assert(public_key == result_public_key.unwrap());

    let hi_2 = b256::zero();
    let lo_2 = 0x44ac566bd156b4fc71a4a4cb2655d3da360c695edb27dc3b64d621e122fea23d;
    let msg_hash_2 = 0x1e45523606c96c98ba970ff7cf9511fab8b25e1bcd52ced30b81df1e4a9c4323;
    let signature_2 = Secp256r1::from((hi_2, lo_2));
    let message_2 = Message::from(msg_hash_2);

    let result_2 = signature_2.recover(message_2);
    assert(result_2.is_err());
}

#[test]
fn secp256r1_address() {
    let hi = 0xbd0c9b8792876713afa8bf3383eebf31c43437823ed761cc3600d0016de5110c;
    let lo = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    let msg_hash = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    let address = Address::from(0xb4a5fabee8cc852084b71f17107e9c18d682033a58967027af0ab01edf2f9a6a);
    let signature = Secp256r1::from((hi, lo));
    let message = Message::from(msg_hash);

    // A recovered Fuel address.
    let result_address = signature.address(message);
    assert(result_address.is_ok());
    assert(result_address.unwrap() == address);

    let hi_2 = b256::zero();
    let lo_2 = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    let msg_hash_2 = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    let signature_2 = Secp256r1::from((hi_2, lo_2));
    let message_2 = Message::from(msg_hash_2);

    let result_2 = signature_2.address(message_2);
    assert(result_2.is_err());
}

#[test]
fn secp256r1_evm_address() {
    let hi_1 = 0x62CDC20C0AB6AA7B91E63DA9917792473F55A6F15006BC99DD4E29420084A3CC;
    let lo_1 = 0xF4D99AF28F9D6BD96BDAAB83BFED99212AC3C7D06810E33FBB14C4F29B635414;
    let msg_hash_1 = 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563;
    let expected_evm_address = EvmAddress::from(0x000000000000000000000000408eb2d97ef0beda0a33848d9e052066667cb00a);
    let signature_1 = Secp256r1::from((hi_1, lo_1));
    let message_1 = Message::from(msg_hash_1);

    let result_1 = signature_1.evm_address(message_1);
    assert(result_1.is_ok());
    assert(result_1.unwrap() == expected_evm_address);

    let hi_2 = b256::zero();
    let lo_2 = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    let msg_hash_2 = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    let signature_2 = Secp256r1::from((hi_2, lo_2));
    let message_2 = Message::from(msg_hash_2);

    let result_2 = signature_2.evm_address(message_2);
    assert(result_2.is_err());
}

#[test]
fn secp256r1_verify() {
    let hi = 0xbd0c9b8792876712afadbff382e1bf31c44437823ed761cc3600d0016de511ac;
    let lo = 0x44ac566bd156b4fc71a4a4cb2655d3da360c695edb27dc3b64d621e122fea23d;
    let msg_hash = 0x1e45523606c96c98ba970ff7cf9511fab8b25e1bcd52ced30b81df1e4a9c4323;
    let pub_hi = 0xd6ea577a54ae42411fbc78d686d4abba2150ca83540528e4b868002e346004b2;
    let pub_lo = 0x62660ecce5979493fe5684526e8e00875b948e507a89a47096bc84064a175452;
    let signature = Secp256r1::from((hi, lo));
    let public_key = PublicKey::from((pub_hi, pub_lo));
    let message = Message::from(msg_hash);

    // A recovered public key pair.
    let result = signature.verify(public_key, message);
    assert(result.is_ok());

    let hi_2 = 0xbd0c9b8792876712afadbff382e1bf31c44437823ed761cc3600d0016de511ac;
    let lo_2 = 0x44ac566bd156b4fc71a4a4cb2655d3da360c695edb27dc3b64d621e122fea23d;
    let msg_hash_2 = 0x1e45523606c96c98ba970ff7cf9511fab8b25e1bcd52ced30b81df1e4a9c4323;
    let pub_hi_2 = b256::zero();
    let pub_lo_2 = 0x62660ecce5979493fe5684526e8e00875b948e507a89a47096bc84064a175452;
    let signature_2 = Secp256r1::from((hi_2, lo_2));
    let public_key_2 = PublicKey::from((pub_hi_2, pub_lo_2));
    let message_2 = Message::from(msg_hash_2);

    let result_2 = signature_2.verify(public_key_2, message_2);
    assert(result_2.is_err());

    let hi_3 = 0xbd0c9b8792876712afadbff382e1bf31c44437823ed761cc3600d0016de511ac;
    let lo_3 = 0x44ac566bd156b4fc71a4a4cb2655d3da360c695edb27dc3b64d621e122fea23d;
    let msg_hash_3 = 0x1e45523606c96c98ba970ff7cf9511fab8b25e1bcd52ced30b81df1e4a9c4323;
    let pub_3 = b256::zero();
    let signature_3 = Secp256r1::from((hi_3, lo_3));
    let public_key_3 = PublicKey::from(pub_3);
    let message_3 = Message::from(msg_hash_3);

    let result_3 = signature_3.verify(public_key_3, message_3);
    assert(result_3.is_err());
}

#[test]
fn secp256r1_verify_address() {
    let hi = 0xbd0c9b8792876713afa8bf3383eebf31c43437823ed761cc3600d0016de5110c;
    let lo = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    let msg_hash = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    let address = Address::from(0xb4a5fabee8cc852084b71f17107e9c18d682033a58967027af0ab01edf2f9a6a);
    let signature = Secp256r1::from((hi, lo));
    let message = Message::from(msg_hash);

    // A recovered Fuel address.
    let result = signature.verify_address(address, message);
    assert(result.is_ok());

    let hi_2 = 0xbd0c9b8792876713afa8bf3383eebf31c43437823ed761cc3600d0016de5110c;
    let lo_2 = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    let msg_hash_2 = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    let address_2 = Address::zero();
    let signature_2 = Secp256r1::from((hi_2, lo_2));
    let message_2 = Message::from(msg_hash_2);

    // A recovered Fuel address.
    let result_2 = signature.verify_address(address_2, message_2);
    assert(result_2.is_err());
}

#[test]
fn secp256r1_verify_evm_address() {
    let hi = 0x62CDC20C0AB6AA7B91E63DA9917792473F55A6F15006BC99DD4E29420084A3CC;
    let lo = 0xF4D99AF28F9D6BD96BDAAB83BFED99212AC3C7D06810E33FBB14C4F29B635414;
    let msg_hash = 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563;
    let address = EvmAddress::from(0x000000000000000000000000408eb2d97ef0beda0a33848d9e052066667cb00a);
    let signature = Secp256r1::from((hi, lo));
    let message = Message::from(msg_hash);

    // A recovered Fuel address.
    let result = signature.verify_evm_address(address, message);
    assert(result.is_ok());

    let hi_2 = 0x62CDC20C0AB6AA7B91E63DA9917792473F55A6F15006BC99DD4E29420084A3CC;
    let lo_2 = 0xF4D99AF28F9D6BD96BDAAB83BFED99212AC3C7D06810E33FBB14C4F29B635414;
    let msg_hash_2 = 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563;
    let address_2 = EvmAddress::zero();
    let signature_2 = Secp256r1::from((hi_2, lo_2));
    let message_2 = Message::from(msg_hash_2);

    // A recovered Fuel address.
    let result_2 = signature_2.verify_evm_address(address_2, message_2);
    assert(result_2.is_err());
}

#[test]
fn secp256r1_from_b512() {
    let b512_1 = B512::from((b256::zero(), b256::zero()));
    let secp256r1_1 = Secp256r1::from(b512_1);
    let mut iter_1 = 0;
    while iter_1 < 64 {
        assert(secp256r1_1.bits()[iter_1] == 0u8);
        iter_1 += 1;
    }

    let b512_2 = B512::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let secp256r1_2 = Secp256r1::from(b512_2);
    assert(secp256r1_2.bits()[63] == 1u8);
    let mut iter_2 = 0;
    while iter_2 < 63 {
        assert(secp256r1_2.bits()[iter_2] == 0u8);
        iter_2 += 1;
    }

    let b512_3 = B512::from((b256::max(), b256::max()));
    let secp256r1_3 = Secp256r1::from(b512_3);
    let mut iter_3 = 0;
    while iter_3 < 64 {
        assert(secp256r1_3.bits()[iter_3] == 255u8);
        iter_3 += 1;
    }
}

#[test]
fn secp256r1_from_b256_tuple() {
    let secp256r1_1 = Secp256r1::from((b256::zero(), b256::zero()));
    let mut iter_1 = 0;
    while iter_1 < 64 {
        assert(secp256r1_1.bits()[iter_1] == 0u8);
        iter_1 += 1;
    }

    let secp256r1_2 = Secp256r1::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    assert(secp256r1_2.bits()[63] == 1u8);
    let mut iter_2 = 0;
    while iter_2 < 63 {
        assert(secp256r1_2.bits()[iter_2] == 0u8);
        iter_2 += 1;
    }

    let secp256r1_3 = Secp256r1::from((b256::max(), b256::max()));
    let mut iter_3 = 0;
    while iter_3 < 64 {
        assert(secp256r1_3.bits()[iter_3] == 255u8);
        iter_3 += 1;
    }
}

#[test]
fn secp256r1_from_u8_array() {
    let array_1 = [
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8,
    ];
    let secp256r1_1 = Secp256r1::from(array_1);
    let mut iter_1 = 0;
    while iter_1 < 64 {
        assert(secp256r1_1.bits()[iter_1] == 0u8);
        iter_1 += 1;
    }

    let array_2 = [
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 1u8,
    ];
    let secp256r1_2 = Secp256r1::from(array_2);
    assert(secp256r1_2.bits()[63] == 1u8);
    let mut iter_2 = 0;
    while iter_2 < 63 {
        assert(secp256r1_2.bits()[iter_2] == 0u8);
        iter_2 += 1;
    }

    let array_3 = [
        255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
        255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
        255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
        255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
        255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
        255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
    ];
    let secp256r1_3 = Secp256r1::from(array_3);
    let mut iter_3 = 0;
    while iter_3 < 64 {
        assert(secp256r1_3.bits()[iter_3] == 255u8);
        iter_3 += 1;
    }
}

#[test]
fn secp256r1_try_from_bytes() {
    let b256_tuple_1 = (b256::zero(), b256::zero());
    let bytes_1 = Bytes::from(raw_slice::from_parts::<u8>(__addr_of(b256_tuple_1), 64));
    let secp256r1_1 = Secp256r1::try_from(bytes_1).unwrap();
    let mut iter_1 = 0;
    while iter_1 < 64 {
        assert(secp256r1_1.bits()[iter_1] == 0u8);
        iter_1 += 1;
    }

    let b256_tuple_2 = (
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    );
    let bytes_2 = Bytes::from(raw_slice::from_parts::<u8>(__addr_of(b256_tuple_2), 64));
    let secp256r1_2 = Secp256r1::try_from(bytes_2).unwrap();
    assert(secp256r1_2.bits()[63] == 1u8);
    let mut iter_2 = 0;
    while iter_2 < 63 {
        assert(secp256r1_2.bits()[iter_2] == 0u8);
        iter_2 += 1;
    }

    let b256_tuple_3 = (b256::max(), b256::max());
    let bytes_3 = Bytes::from(raw_slice::from_parts::<u8>(__addr_of(b256_tuple_3), 64));
    let secp256r1_3 = Secp256r1::try_from(bytes_3).unwrap();
    let mut iter_3 = 0;
    while iter_3 < 64 {
        assert(secp256r1_3.bits()[iter_3] == 255u8);
        iter_3 += 1;
    }

    let bytes_4 = Bytes::new();
    let secp256r1_4 = Secp256r1::try_from(bytes_4);
    assert(secp256r1_4.is_none());

    let mut bytes_5 = Bytes::new();
    bytes_5.push(0u8);
    bytes_5.push(0u8);
    bytes_5.push(0u8);
    let secp256r1_5 = Secp256r1::try_from(bytes_5);
    assert(secp256r1_5.is_none());
}

#[test]
fn secp256r1_into_b512() {
    let b512_1 = B512::from((b256::zero(), b256::zero()));
    let secp256r1_1 = Secp256r1::from(b512_1);
    assert(<Secp256r1 as Into<B512>>::into(secp256r1_1) == b512_1);

    let b512_2 = B512::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let secp256r1_2 = Secp256r1::from(b512_2);
    assert(<Secp256r1 as Into<B512>>::into(secp256r1_2) == b512_2);

    let b512_3 = B512::from((b256::max(), b256::max()));
    let secp256r1_3 = Secp256r1::from(b512_3);
    assert(<Secp256r1 as Into<B512>>::into(secp256r1_3) == b512_3);
}

#[test]
fn secp256r1_into_b256_tuple() {
    let secp256r1_1 = Secp256r1::from((b256::zero(), b256::zero()));
    let (result_1_1, result_2_1) = <Secp256r1 as Into<(b256, b256)>>::into(secp256r1_1);
    assert(result_1_1 == b256::zero());
    assert(result_2_1 == b256::zero());

    let secp256r1_2 = Secp256r1::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let (result_1_2, result_2_2) = <Secp256r1 as Into<(b256, b256)>>::into(secp256r1_2);
    assert(result_1_2 == b256::zero());
    assert(
        result_2_2 == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );

    let secp256r1_3 = Secp256r1::from((b256::max(), b256::max()));
    let (result_1_3, result_2_3) = <Secp256r1 as Into<(b256, b256)>>::into(secp256r1_3);
    assert(result_1_3 == b256::max());
    assert(result_2_3 == b256::max());
}

#[test]
fn secp256r1_into_bytes() {
    let secp256r1_1 = Secp256r1::from((b256::zero(), b256::zero()));
    let bytes_result_1 = <Secp256r1 as Into<Bytes>>::into(secp256r1_1);
    let mut iter_1 = 0;
    while iter_1 < 64 {
        assert(bytes_result_1.get(iter_1).unwrap() == 0u8);
        iter_1 += 1;
    }

    let secp256r1_2 = Secp256r1::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let bytes_result_2 = <Secp256r1 as Into<Bytes>>::into(secp256r1_2);
    assert(bytes_result_2.get(63).unwrap() == 1u8);
    let mut iter_2 = 0;
    while iter_2 < 63 {
        assert(bytes_result_2.get(iter_2).unwrap() == 0u8);
        iter_2 += 1;
    }

    let secp256r1_3 = Secp256r1::from((b256::max(), b256::max()));
    let bytes_result_3 = <Secp256r1 as Into<Bytes>>::into(secp256r1_3);
    let mut iter_3 = 0;
    while iter_3 < 64 {
        assert(bytes_result_3.get(iter_3).unwrap() == 255u8);
        iter_3 += 1;
    }
}

#[test]
fn secp256r1_eq() {
    let secp256r1_1 = Secp256r1::from((b256::zero(), b256::zero()));
    let secp256r1_2 = Secp256r1::from((b256::zero(), b256::zero()));
    let secp256r1_3 = Secp256r1::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let secp256r1_4 = Secp256r1::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let secp256r1_5 = Secp256r1::from((b256::max(), b256::max()));
    let secp256r1_6 = Secp256r1::from((b256::max(), b256::max()));

    assert(secp256r1_1 == secp256r1_2);
    assert(secp256r1_3 == secp256r1_4);
    assert(secp256r1_5 == secp256r1_6);

    assert(secp256r1_1 != secp256r1_3);
    assert(secp256r1_1 != secp256r1_5);

    assert(secp256r1_3 != secp256r1_5);
}

#[test]
fn secp256r1_hash() {
    let secp256r1 = Secp256r1::from((b256::zero(), b256::zero()));
    let hash = sha256(secp256r1);
    assert(hash == 0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b);
}

#[test]
fn secp256r1_codec() {
    let secp256r1 = Secp256r1::from((b256::zero(), b256::zero()));
    log(secp256r1);
}
