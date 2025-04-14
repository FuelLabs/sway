library;

use std::{
    b512::B512,
    bytes::Bytes,
    crypto::{
        message::*,
        public_key::*,
        secp256k1::*,
    },
    hash::{
        Hash,
        sha256,
    },
    vm::evm::evm_address::EvmAddress,
};

#[test]
fn secp256k1_new() {
    let new_secp256k1 = Secp256k1::new();
    let mut iter = 0;
    while iter < 64 {
        assert(new_secp256k1.bits()[iter] == 0u8);
        iter += 1;
    }
}

#[test]
fn secp256k1_bits() {
    let new_secp256k1 = Secp256k1::new();
    let secp256k1_bits = new_secp256k1.bits();
    let mut iter = 0;
    while iter < 64 {
        assert(secp256k1_bits[iter] == 0u8);
        iter += 1;
    }
}

#[test]
fn secp256k1_recover() {
    let hi = 0x61f3caf4c0912cec69ff0b226638d397115c623a7f057914d48a7e4daf1cf6d8;
    let lo = 0x2555de81cd3a40382d3d64eb1c77e463eea5a76d65ec85f283e0b3d568352678;
    let msg_hash = 0xa13f4ab54057ce064d3dd97ac3ff30ed704e73956896c03650fe59b1a561fe15;
    let pub_hi = 0x41a55558a3486b6ee3878f55f16879c0798afd772c1506de44aba90d29b6e65c;
    let pub_lo = 0x341ca2e0a3d5827e78d838e35b29bebe2a39ac30b58999e1138c9467bf859965;
    let signature: Secp256k1 = Secp256k1::from((hi, lo));
    let public_key = PublicKey::from((pub_hi, pub_lo));
    let message = Message::from(msg_hash);

    // A recovered public key pair.
    let result_public_key = signature.recover(message);
    assert(result_public_key.is_ok());
    assert(public_key == result_public_key.unwrap());

    let hi_2 = b256::zero();
    let lo_2 = 0x2555de81cd3a40382d3d64eb1c77e463eea5a76d65ec85f283e0b3d568352678;
    let msg_hash_2 = 0xa13f4ab54057ce064d3dd97ac3ff30ed704e73956896c03650fe59b1a561fe15;
    let signature_2 = Secp256k1::from((hi_2, lo_2));
    let message_2 = Message::from(msg_hash_2);

    let result_2 = signature_2.recover(message_2);
    assert(result_2.is_err());
}

#[test]
fn secp256k1_address() {
    let hi = 0x61f3caf4c0912cec69ff0b226638d397115c623a7f057914d48a7e4daf1cf6d8;
    let lo = 0x2555de81cd3a40382d3d64eb1c77e463eea5a76d65ec85f283e0b3d568352678;
    let msg_hash = 0xa13f4ab54057ce064d3dd97ac3ff30ed704e73956896c03650fe59b1a561fe15;
    let address = Address::from(0x02844f00cce0f608fa3f0f7408bec96bfd757891a6fda6e1fa0f510398304881);
    let signature = Secp256k1::from((hi, lo));
    let message = Message::from(msg_hash);

    // A recovered Fuel address.
    let result_address = signature.address(message);
    assert(result_address.is_ok());
    assert(result_address.unwrap() == address);

    let hi_2 = b256::zero();
    let lo_2 = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    let msg_hash_2 = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    let signature_2 = Secp256k1::from((hi_2, lo_2));
    let message_2 = Message::from(msg_hash_2);

    let result_2 = signature_2.address(message_2);
    assert(result_2.is_err());
}

#[test]
fn secp256k1_evm_address() {
    let hi_1 = 0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c;
    let lo_1 = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    let msg_hash_1 = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    let expected_evm_address = EvmAddress::from(0x0000000000000000000000000ec44cf95ce5051ef590e6d420f8e722dd160ecb);
    let signature_1 = Secp256k1::from((hi_1, lo_1));
    let message_1 = Message::from(msg_hash_1);

    let result_1 = signature_1.evm_address(message_1);
    assert(result_1.is_ok());
    assert(result_1.unwrap() == expected_evm_address);

    let hi_2 = 0xbd0c9b8792876713afa8bf1383eebf31c43437823ed761cc3600d0016de5110c;
    let lo_2 = 0xee45573606c96c98ba170ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    let msg_hash_2 = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52cad30b89df1e4a9c4323;
    let signature_2 = Secp256k1::from((hi_2, lo_2));
    let message_2 = Message::from(msg_hash_2);

    let result_2 = signature_2.evm_address(message_2);
    assert(result_2.is_err());
}

#[test]
fn secp256k1_verify() {
    let hi = 0x61f3caf4c0912cec69ff0b226638d397115c623a7f057914d48a7e4daf1cf6d8;
    let lo = 0x2555de81cd3a40382d3d64eb1c77e463eea5a76d65ec85f283e0b3d568352678;
    let msg_hash = 0xa13f4ab54057ce064d3dd97ac3ff30ed704e73956896c03650fe59b1a561fe15;
    let pub_hi = 0x41a55558a3486b6ee3878f55f16879c0798afd772c1506de44aba90d29b6e65c;
    let pub_lo = 0x341ca2e0a3d5827e78d838e35b29bebe2a39ac30b58999e1138c9467bf859965;
    let signature = Secp256k1::from((hi, lo));
    let public_key = PublicKey::from((pub_hi, pub_lo));
    let message = Message::from(msg_hash);

    // A recovered public key pair.
    let result = signature.verify(public_key, message);
    assert(result.is_ok());

    let hi_2 = b256::zero();
    let lo_2 = 0x2555de81cd3a40382d3d64eb1c77e463eea5a76d65ec85f283e0b3d568352678;
    let msg_hash_2 = 0xa13f4ab54057ce064d3dd97ac3ff30ed704e73956896c03650fe59b1a561fe15;
    let pub_hi_2 = 0x41a55558a3486b6ee3878f55f16879c0798afd772c1506de44aba90d29b6e65c;
    let pub_lo_2 = 0x62660ecce5979493fe5684526e8e00875b948e507a89a47096bc84064a175452;
    let signature_2 = Secp256k1::from((hi_2, lo_2));
    let public_key_2 = PublicKey::from((pub_hi_2, pub_lo_2));
    let message_2 = Message::from(msg_hash_2);

    let result_2 = signature_2.verify(public_key_2, message_2);
    assert(result_2.is_err());

    let hi_3 = 0xbd0c9b8792876712afadbff382e1bf31c44437823ed761cc3600d0016de511ac;
    let lo_3 = 0x44ac566bd156b4fc71a4a4cb2655d3da360c695edb27dc3b64d621e122fea23d;
    let msg_hash_3 = 0x1e45523606c96c98ba970ff7cf9511fab8b25e1bcd52ced30b81df1e4a9c4323;
    let pub_3 = b256::zero();
    let signature_3 = Secp256k1::from((hi_3, lo_3));
    let public_key_3 = PublicKey::from(pub_3);
    let message_3 = Message::from(msg_hash_3);

    let result_3 = signature_3.verify(public_key_3, message_3);
    assert(result_3.is_err());
}

#[test]
fn secp256k1_verify_address() {
    let hi = 0x61f3caf4c0912cec69ff0b226638d397115c623a7f057914d48a7e4daf1cf6d8;
    let lo = 0x2555de81cd3a40382d3d64eb1c77e463eea5a76d65ec85f283e0b3d568352678;
    let msg_hash = 0xa13f4ab54057ce064d3dd97ac3ff30ed704e73956896c03650fe59b1a561fe15;
    let address = Address::from(0x02844f00cce0f608fa3f0f7408bec96bfd757891a6fda6e1fa0f510398304881);
    let signature = Secp256k1::from((hi, lo));
    let message = Message::from(msg_hash);

    // A recovered Fuel address.
    let result = signature.verify_address(address, message);
    assert(result.is_ok());

    let hi_2 = 0x61f3caf4c0912cec69ff0b226638d397115c623a7f057914d48a7e4daf1cf6d8;
    let lo_2 = 0x2555de81cd3a40382d3d64eb1c77e463eea5a76d65ec85f283e0b3d568352678;
    let msg_hash_2 = 0xa13f4ab54057ce064d3dd97ac3ff30ed704e73956896c03650fe59b1a561fe15;
    let address_2 = Address::zero();
    let signature_2 = Secp256k1::from((hi_2, lo_2));
    let message_2 = Message::from(msg_hash_2);

    // A recovered Fuel address.
    let result_2 = signature.verify_address(address_2, message_2);
    assert(result_2.is_err());
}

#[test]
fn secp256k1_verify_evm_address() {
    let hi = 0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c;
    let lo = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    let msg_hash = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    let address = EvmAddress::from(0x0000000000000000000000000ec44cf95ce5051ef590e6d420f8e722dd160ecb);
    let signature = Secp256k1::from((hi, lo));
    let message = Message::from(msg_hash);

    // A recovered EVM address.
    let result = signature.verify_evm_address(address, message);
    assert(result.is_ok());

    let hi_2 = 0xbd0c9b8792876713afa8bf3383eebf31c43437823ed761cc3600d0016de5110c;
    let lo_2 = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    let msg_hash_2 = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    let address_2 = EvmAddress::zero();
    let signature_2 = Secp256k1::from((hi_2, lo_2));
    let message_2 = Message::from(msg_hash_2);

    // A recovered Fuel address.
    let result_2 = signature.verify_evm_address(address_2, message_2);
    assert(result_2.is_err());
}

#[test]
fn secp256k1_from_b512() {
    let b512_1 = B512::from((b256::zero(), b256::zero()));
    let secp256k1_1 = Secp256k1::from(b512_1);
    let mut iter_1 = 0;
    while iter_1 < 64 {
        assert(secp256k1_1.bits()[iter_1] == 0u8);
        iter_1 += 1;
    }

    let b512_2 = B512::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let secp256k1_2 = Secp256k1::from(b512_2);
    assert(secp256k1_2.bits()[63] == 1u8);
    let mut iter_2 = 0;
    while iter_2 < 63 {
        assert(secp256k1_2.bits()[iter_2] == 0u8);
        iter_2 += 1;
    }

    let b512_3 = B512::from((b256::max(), b256::max()));
    let secp256k1_3 = Secp256k1::from(b512_3);
    let mut iter_3 = 0;
    while iter_3 < 64 {
        assert(secp256k1_3.bits()[iter_3] == 255u8);
        iter_3 += 1;
    }
}

#[test]
fn secp256k1_from_b256_tuple() {
    let secp256k1_1 = Secp256k1::from((b256::zero(), b256::zero()));
    let mut iter_1 = 0;
    while iter_1 < 64 {
        assert(secp256k1_1.bits()[iter_1] == 0u8);
        iter_1 += 1;
    }

    let secp256k1_2 = Secp256k1::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    assert(secp256k1_2.bits()[63] == 1u8);
    let mut iter_2 = 0;
    while iter_2 < 63 {
        assert(secp256k1_2.bits()[iter_2] == 0u8);
        iter_2 += 1;
    }

    let secp256k1_3 = Secp256k1::from((b256::max(), b256::max()));
    let mut iter_3 = 0;
    while iter_3 < 64 {
        assert(secp256k1_3.bits()[iter_3] == 255u8);
        iter_3 += 1;
    }
}

#[test]
fn secp256k1_from_u8_array() {
    let array_1 = [
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8,
    ];
    let secp256k1_1 = Secp256k1::from(array_1);
    let mut iter_1 = 0;
    while iter_1 < 64 {
        assert(secp256k1_1.bits()[iter_1] == 0u8);
        iter_1 += 1;
    }

    let array_2 = [
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 1u8,
    ];
    let secp256k1_2 = Secp256k1::from(array_2);
    assert(secp256k1_2.bits()[63] == 1u8);
    let mut iter_2 = 0;
    while iter_2 < 63 {
        assert(secp256k1_2.bits()[iter_2] == 0u8);
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
    let secp256k1_3 = Secp256k1::from(array_3);
    let mut iter_3 = 0;
    while iter_3 < 64 {
        assert(secp256k1_3.bits()[iter_3] == 255u8);
        iter_3 += 1;
    }
}

#[test]
fn secp256k1_try_from_bytes() {
    let b256_tuple_1 = (b256::zero(), b256::zero());
    let bytes_1 = Bytes::from(raw_slice::from_parts::<u8>(__addr_of(b256_tuple_1), 64));
    let secp256k1_1 = Secp256k1::try_from(bytes_1).unwrap();
    let mut iter_1 = 0;
    while iter_1 < 64 {
        assert(secp256k1_1.bits()[iter_1] == 0u8);
        iter_1 += 1;
    }

    let b256_tuple_2 = (
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    );
    let bytes_2 = Bytes::from(raw_slice::from_parts::<u8>(__addr_of(b256_tuple_2), 64));
    let secp256k1_2 = Secp256k1::try_from(bytes_2).unwrap();
    assert(secp256k1_2.bits()[63] == 1u8);
    let mut iter_2 = 0;
    while iter_2 < 63 {
        assert(secp256k1_2.bits()[iter_2] == 0u8);
        iter_2 += 1;
    }

    let b256_tuple_3 = (b256::max(), b256::max());
    let bytes_3 = Bytes::from(raw_slice::from_parts::<u8>(__addr_of(b256_tuple_3), 64));
    let secp256k1_3 = Secp256k1::try_from(bytes_3).unwrap();
    let mut iter_3 = 0;
    while iter_3 < 64 {
        assert(secp256k1_3.bits()[iter_3] == 255u8);
        iter_3 += 1;
    }

    let bytes_4 = Bytes::new();
    let secp256k1_4 = Secp256k1::try_from(bytes_4);
    assert(secp256k1_4.is_none());

    let mut bytes_5 = Bytes::new();
    bytes_5.push(0u8);
    bytes_5.push(0u8);
    bytes_5.push(0u8);
    let secp256k1_5 = Secp256k1::try_from(bytes_5);
    assert(secp256k1_5.is_none());
}

#[test]
fn secp256k1_into_b512() {
    let b512_1 = B512::from((b256::zero(), b256::zero()));
    let secp256k1_1 = Secp256k1::from(b512_1);
    assert(<Secp256k1 as Into<B512>>::into(secp256k1_1) == b512_1);

    let b512_2 = B512::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let secp256k1_2 = Secp256k1::from(b512_2);
    assert(<Secp256k1 as Into<B512>>::into(secp256k1_2) == b512_2);

    let b512_3 = B512::from((b256::max(), b256::max()));
    let secp256k1_3 = Secp256k1::from(b512_3);
    assert(<Secp256k1 as Into<B512>>::into(secp256k1_3) == b512_3);
}

#[test]
fn secp256k1_into_b256_tuple() {
    let secp256k1_1 = Secp256k1::from((b256::zero(), b256::zero()));
    let (result_1_1, result_2_1) = <Secp256k1 as Into<(b256, b256)>>::into(secp256k1_1);
    assert(result_1_1 == b256::zero());
    assert(result_2_1 == b256::zero());

    let secp256k1_2 = Secp256k1::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let (result_1_2, result_2_2) = <Secp256k1 as Into<(b256, b256)>>::into(secp256k1_2);
    assert(result_1_2 == b256::zero());
    assert(
        result_2_2 == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );

    let secp256k1_3 = Secp256k1::from((b256::max(), b256::max()));
    let (result_1_3, result_2_3) = <Secp256k1 as Into<(b256, b256)>>::into(secp256k1_3);
    assert(result_1_3 == b256::max());
    assert(result_2_3 == b256::max());
}

#[test]
fn secp256k1_into_bytes() {
    let secp256k1_1 = Secp256k1::from((b256::zero(), b256::zero()));
    let bytes_result_1 = <Secp256k1 as Into<Bytes>>::into(secp256k1_1);
    let mut iter_1 = 0;
    while iter_1 < 64 {
        assert(bytes_result_1.get(iter_1).unwrap() == 0u8);
        iter_1 += 1;
    }

    let secp256k1_2 = Secp256k1::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let bytes_result_2 = <Secp256k1 as Into<Bytes>>::into(secp256k1_2);
    assert(bytes_result_2.get(63).unwrap() == 1u8);
    let mut iter_2 = 0;
    while iter_2 < 63 {
        assert(bytes_result_2.get(iter_2).unwrap() == 0u8);
        iter_2 += 1;
    }

    let secp256k1_3 = Secp256k1::from((b256::max(), b256::max()));
    let bytes_result_3 = <Secp256k1 as Into<Bytes>>::into(secp256k1_3);
    let mut iter_3 = 0;
    while iter_3 < 64 {
        assert(bytes_result_3.get(iter_3).unwrap() == 255u8);
        iter_3 += 1;
    }
}

#[test]
fn secp256k1_eq() {
    let secp256k1_1 = Secp256k1::from((b256::zero(), b256::zero()));
    let secp256k1_2 = Secp256k1::from((b256::zero(), b256::zero()));
    let secp256k1_3 = Secp256k1::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let secp256k1_4 = Secp256k1::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let secp256k1_5 = Secp256k1::from((b256::max(), b256::max()));
    let secp256k1_6 = Secp256k1::from((b256::max(), b256::max()));

    assert(secp256k1_1 == secp256k1_2);
    assert(secp256k1_3 == secp256k1_4);
    assert(secp256k1_5 == secp256k1_6);

    assert(secp256k1_1 != secp256k1_3);
    assert(secp256k1_1 != secp256k1_5);

    assert(secp256k1_3 != secp256k1_5);
}

#[test]
fn secp256k1_hash() {
    let secp256k1 = Secp256k1::from((b256::zero(), b256::zero()));
    let hash = sha256(secp256k1);
    assert(hash == 0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b);
}

#[test]
fn secp256k1_codec() {
    let secp256k1 = Secp256k1::from((b256::zero(), b256::zero()));
    log(secp256k1);
}
