library;

use std::{
    b512::B512,
    bytes::Bytes,
    crypto::{
        ed25519::*,
        message::*,
        public_key::*,
    },
    hash::{
        Hash,
        sha256,
    },
    vm::evm::evm_address::EvmAddress,
};

#[test]
fn ed25519_new() {
    let new_ed25519 = Ed25519::new();
    let mut iter = 0;
    while iter < 64 {
        assert(new_ed25519.bits()[iter] == 0u8);
        iter += 1;
    }
}

#[test]
fn ed25519_bits() {
    let new_ed25519 = Ed25519::new();
    let ed25519_bits = new_ed25519.bits();
    let mut iter = 0;
    while iter < 64 {
        assert(ed25519_bits[iter] == 0u8);
        iter += 1;
    }
}

#[test]
fn ed25519__verify() {
    let pub_key = 0x314fa58689bbe1da2430517de2d772b384a1c1d2e9cb87e73c6afcf246045b10;
    let msg = b256::zero();
    let msg_hash = sha256(msg);
    let hi = 0xf38cef9361894be6c6e0eddec28a663d099d7ddff17c8077a1447d7ecb4e6545;
    let lo = 0xf5084560039486d3462dd65a40c80a74709b2f06d450ffc5dc00345c6b2cdd00;
    let public_key: PublicKey = PublicKey::from(pub_key);
    let signature: Ed25519 = Ed25519::from((hi, lo));
    let message: Message = Message::from(msg_hash);

    // A verified public key with signature 
    let verified = signature.verify(public_key, message);
    assert(verified.is_ok());

    let pub_key_2 = 0x314fa58689bbe1da2430517de2d772b384a1c1d2e9cb87e73c6afcf246045b10;
    let msg_2 = b256::zero();
    let msg_hash_2 = sha256(msg_2);
    let hi_2 = b256::zero();
    let lo_2 = 0xf5084560039486d3462dd65a40c80a74709b2f06d450ffc5dc00345c6b2cdd00;
    let public_key_2: PublicKey = PublicKey::from(pub_key_2);
    let signature_2: Ed25519 = Ed25519::from((hi_2, lo_2));
    let message_2: Message = Message::from(msg_hash_2);

    let verified_2 = signature_2.verify(public_key_2, message_2);
    assert(verified_2.is_err());

    let pub_key_3 = 0x314fa58689bbe1da2430517de2d772b384a1c1d2e9cb87e73c6afcf246045b10;
    let msg_3 = b256::zero();
    let msg_hash_3 = sha256(msg_3);
    let hi_3 = 0xf38cef9361894be6c6e0eddec28a663d099d7ddff17c8077a1447d7ecb4e6545;
    let lo_3 = 0xf5084560039486d3462dd65a40c80a74709b2f06d450ffc5dc00345c6b2cdd00;
    let public_key_3: PublicKey = PublicKey::from((pub_key_3, b256::zero()));
    let signature_3: Ed25519 = Ed25519::from((hi_3, lo_3));
    let message_3: Message = Message::from(msg_hash_3);

    // A verified public key with signature 
    let verified_3 = signature_3.verify(public_key_3, message_3);
    assert(verified_3.is_err());
}

#[test]
fn ed25519_from_b512() {
    let b512_1 = B512::from((b256::zero(), b256::zero()));
    let ed25519_1 = Ed25519::from(b512_1);
    let mut iter_1 = 0;
    while iter_1 < 64 {
        assert(ed25519_1.bits()[iter_1] == 0u8);
        iter_1 += 1;
    }

    let b512_2 = B512::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let ed25519_2 = Ed25519::from(b512_2);
    assert(ed25519_2.bits()[63] == 1u8);
    let mut iter_2 = 0;
    while iter_2 < 63 {
        assert(ed25519_2.bits()[iter_2] == 0u8);
        iter_2 += 1;
    }

    let b512_3 = B512::from((b256::max(), b256::max()));
    let ed25519_3 = Ed25519::from(b512_3);
    let mut iter_3 = 0;
    while iter_3 < 64 {
        assert(ed25519_3.bits()[iter_3] == 255u8);
        iter_3 += 1;
    }
}

#[test]
fn ed25519_from_b256_tuple() {
    let ed25519_1 = Ed25519::from((b256::zero(), b256::zero()));
    let mut iter_1 = 0;
    while iter_1 < 64 {
        assert(ed25519_1.bits()[iter_1] == 0u8);
        iter_1 += 1;
    }

    let ed25519_2 = Ed25519::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    assert(ed25519_2.bits()[63] == 1u8);
    let mut iter_2 = 0;
    while iter_2 < 63 {
        assert(ed25519_2.bits()[iter_2] == 0u8);
        iter_2 += 1;
    }

    let ed25519_3 = Ed25519::from((b256::max(), b256::max()));
    let mut iter_3 = 0;
    while iter_3 < 64 {
        assert(ed25519_3.bits()[iter_3] == 255u8);
        iter_3 += 1;
    }
}

// TODO: Enable this test once https://github.com/FuelLabs/sway/issues/7157 is fixed.
// #[test]
// fn ed25519_from_u8_array() {
//     let array_1 = [
//         0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
//         0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
//         0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
//         0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
//         0u8, 0u8, 0u8, 0u8,
//     ];
//     let ed25519_1 = Ed25519::from(array_1);
//     let mut iter_1 = 0;
//     while iter_1 < 64 {
//         assert(ed25519_1.bits()[iter_1] == 0u8);
//         iter_1 += 1;
//     }

//     let array_2 = [
//         0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
//         0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
//         0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
//         0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
//         0u8, 0u8, 0u8, 1u8,
//     ];
//     let ed25519_2 = Ed25519::from(array_2);
//     assert(ed25519_2.bits()[63] == 1u8);
//     let mut iter_2 = 0;
//     while iter_2 < 63 {
//         assert(ed25519_2.bits()[iter_2] == 0u8);
//         iter_2 += 1;
//     }

//     let array_3 = [
//         255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
//         255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
//         255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
//         255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
//         255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
//         255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
//     ];
//     let ed25519_3 = Ed25519::from(array_3);
//     let mut iter_3 = 0;
//     while iter_3 < 64 {
//         assert(ed25519_3.bits()[iter_3] == 255u8);
//         iter_3 += 1;
//     }
// }

#[test]
fn ed25519_try_from_bytes() {
    let b256_tuple_1 = (b256::zero(), b256::zero());
    let bytes_1 = Bytes::from(raw_slice::from_parts::<u8>(__addr_of(b256_tuple_1), 64));
    let ed25519_1 = Ed25519::try_from(bytes_1).unwrap();
    let mut iter_1 = 0;
    while iter_1 < 64 {
        assert(ed25519_1.bits()[iter_1] == 0u8);
        iter_1 += 1;
    }

    let b256_tuple_2 = (
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    );
    let bytes_2 = Bytes::from(raw_slice::from_parts::<u8>(__addr_of(b256_tuple_2), 64));
    let ed25519_2 = Ed25519::try_from(bytes_2).unwrap();
    assert(ed25519_2.bits()[63] == 1u8);
    let mut iter_2 = 0;
    while iter_2 < 63 {
        assert(ed25519_2.bits()[iter_2] == 0u8);
        iter_2 += 1;
    }

    let b256_tuple_3 = (b256::max(), b256::max());
    let bytes_3 = Bytes::from(raw_slice::from_parts::<u8>(__addr_of(b256_tuple_3), 64));
    let ed25519_3 = Ed25519::try_from(bytes_3).unwrap();
    let mut iter_3 = 0;
    while iter_3 < 64 {
        assert(ed25519_3.bits()[iter_3] == 255u8);
        iter_3 += 1;
    }

    let bytes_4 = Bytes::new();
    let ed25519_4 = Ed25519::try_from(bytes_4);
    assert(ed25519_4.is_none());

    let mut bytes_5 = Bytes::new();
    bytes_5.push(0u8);
    bytes_5.push(0u8);
    bytes_5.push(0u8);
    let ed25519_5 = Ed25519::try_from(bytes_5);
    assert(ed25519_5.is_none());
}

#[test]
fn ed25519_into_b512() {
    let b512_1 = B512::from((b256::zero(), b256::zero()));
    let ed25519_1 = Ed25519::from(b512_1);
    assert(<Ed25519 as Into<B512>>::into(ed25519_1) == b512_1);

    let b512_2 = B512::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let ed25519_2 = Ed25519::from(b512_2);
    assert(<Ed25519 as Into<B512>>::into(ed25519_2) == b512_2);

    let b512_3 = B512::from((b256::max(), b256::max()));
    let ed25519_3 = Ed25519::from(b512_3);
    assert(<Ed25519 as Into<B512>>::into(ed25519_3) == b512_3);
}

#[test]
fn ed25519_into_b256() {
    let ed25519_1 = Ed25519::from((b256::zero(), b256::zero()));
    let (result_1_1, result_2_1) = <Ed25519 as Into<(b256, b256)>>::into(ed25519_1);
    assert(result_1_1 == b256::zero());
    assert(result_2_1 == b256::zero());

    let ed25519_2 = Ed25519::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let (result_1_2, result_2_2): (b256, b256) = <Ed25519 as Into<(b256, b256)>>::into(ed25519_2);
    assert(result_1_2 == b256::zero());
    assert(
        result_2_2 == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );

    let ed25519_3 = Ed25519::from((b256::max(), b256::max()));
    let (result_1_3, result_2_3): (b256, b256) = <Ed25519 as Into<(b256, b256)>>::into(ed25519_3);
    assert(result_1_3 == b256::max());
    assert(result_2_3 == b256::max());
}

#[test]
fn ed25519_into_bytes() {
    let ed25519_1 = Ed25519::from((b256::zero(), b256::zero()));
    let bytes_result_1: Bytes = <Ed25519 as Into<Bytes>>::into(ed25519_1);
    let mut iter_1 = 0;
    while iter_1 < 64 {
        assert(bytes_result_1.get(iter_1).unwrap() == 0u8);
        iter_1 += 1;
    }

    let ed25519_2 = Ed25519::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let bytes_result_2: Bytes = <Ed25519 as Into<Bytes>>::into(ed25519_2);
    assert(bytes_result_2.get(63).unwrap() == 1u8);
    let mut iter_2 = 0;
    while iter_2 < 63 {
        assert(bytes_result_2.get(iter_2).unwrap() == 0u8);
        iter_2 += 1;
    }

    let ed25519_3 = Ed25519::from((b256::max(), b256::max()));
    let bytes_result_3: Bytes = <Ed25519 as Into<Bytes>>::into(ed25519_3);
    let mut iter_3 = 0;
    while iter_3 < 64 {
        assert(bytes_result_3.get(iter_3).unwrap() == 255u8);
        iter_3 += 1;
    }
}

#[test]
fn ed25519_eq() {
    let ed25519_1 = Ed25519::from((b256::zero(), b256::zero()));
    let ed25519_2 = Ed25519::from((b256::zero(), b256::zero()));
    let ed25519_3 = Ed25519::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let ed25519_4 = Ed25519::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let ed25519_5 = Ed25519::from((b256::max(), b256::max()));
    let ed25519_6 = Ed25519::from((b256::max(), b256::max()));

    assert(ed25519_1 == ed25519_2);
    assert(ed25519_3 == ed25519_4);
    assert(ed25519_5 == ed25519_6);

    assert(ed25519_1 != ed25519_3);
    assert(ed25519_1 != ed25519_5);

    assert(ed25519_3 != ed25519_5);
}

#[test]
fn ed25519_hash() {
    let ed25519 = Ed25519::from((b256::zero(), b256::zero()));
    let hash = sha256(ed25519);
    assert(hash == 0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b);
}

#[test]
fn ed25519_codec() {
    let ed25519 = Ed25519::from((b256::zero(), b256::zero()));
    log(ed25519);
}
