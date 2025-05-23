library;

use std::{b512::B512, bytes::Bytes, crypto::public_key::*, hash::{Hash, sha256}};

#[test]
fn public_key_new() {
    let new_public_key = PublicKey::new();

    assert(new_public_key.bytes().len() == 0);
    assert(new_public_key.bytes().capacity() == 0);
}

#[test]
fn public_key_bytes() {
    let new_public_key = PublicKey::new();
    let new_public_key_bytes = new_public_key.bytes();
    assert(new_public_key_bytes.len() == 0);
    assert(new_public_key_bytes.capacity() == 0);

    let other_public_key = PublicKey::from(b256::max());
    let other_public_key_bytes = other_public_key.bytes();
    assert(other_public_key_bytes.len() == 32);
    assert(other_public_key_bytes.get(0).unwrap() == 255u8);
    assert(other_public_key_bytes.get(1).unwrap() == 255u8);
    assert(other_public_key_bytes.get(2).unwrap() == 255u8);
}

#[test]
fn public_key_is_zero() {
    let new_public_key = PublicKey::new();
    assert(new_public_key.is_zero());

    let b256_public_key = PublicKey::from(b256::zero());
    assert(b256_public_key.is_zero());

    let b256_tuple_public_key = PublicKey::from((b256::zero(), b256::zero()));
    assert(b256_tuple_public_key.is_zero());
}

#[test]
fn public_key_from_b512() {
    let b512_1 = B512::from((b256::zero(), b256::zero()));
    let public_key_1 = PublicKey::from(b512_1);
    assert(public_key_1.bytes().len() == 64);
    let mut iter_1 = 0;
    while iter_1 < public_key_1.bytes().len() {
        assert(public_key_1.bytes().get(iter_1).unwrap() == 0u8);
        iter_1 += 1;
    }

    let b512_2 = B512::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let public_key_2 = PublicKey::from(b512_2);
    assert(public_key_2.bytes().len() == 64);
    assert(public_key_2.bytes().get(63).unwrap() == 1u8);
    let mut iter_2 = 0;
    while iter_2 < public_key_2.bytes().len() - 1 {
        assert(public_key_2.bytes().get(iter_2).unwrap() == 0u8);
        iter_2 += 1;
    }

    let b512_3 = B512::from((b256::max(), b256::max()));
    let public_key_3 = PublicKey::from(b512_3);
    assert(public_key_3.bytes().len() == 64);
    let mut iter_3 = 0;
    while iter_3 < public_key_3.bytes().len() {
        assert(public_key_3.bytes().get(iter_3).unwrap() == 255u8);
        iter_3 += 1;
    }
}

#[test]
fn public_key_from_b256_tuple() {
    let public_key_1 = PublicKey::from((b256::zero(), b256::zero()));
    assert(public_key_1.bytes().len() == 64);
    let mut iter_1 = 0;
    while iter_1 < public_key_1.bytes().len() {
        assert(public_key_1.bytes().get(iter_1).unwrap() == 0u8);
        iter_1 += 1;
    }

    let public_key_2 = PublicKey::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    assert(public_key_2.bytes().len() == 64);
    assert(public_key_2.bytes().get(63).unwrap() == 1u8);
    let mut iter_2 = 0;
    while iter_2 < public_key_2.bytes().len() - 1 {
        assert(public_key_2.bytes().get(iter_2).unwrap() == 0u8);
        iter_2 += 1;
    }

    let public_key_3 = PublicKey::from((b256::max(), b256::max()));
    assert(public_key_3.bytes().len() == 64);
    let mut iter_3 = 0;
    while iter_3 < public_key_3.bytes().len() {
        assert(public_key_3.bytes().get(iter_3).unwrap() == 255u8);
        iter_3 += 1;
    }
}

#[test]
fn public_key_from_b256() {
    let public_key_1 = PublicKey::from(b256::zero());
    assert(public_key_1.bytes().len() == 32);
    let mut iter_1 = 0;
    while iter_1 < public_key_1.bytes().len() {
        assert(public_key_1.bytes().get(iter_1).unwrap() == 0u8);
        iter_1 += 1;
    }

    let public_key_2 = PublicKey::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    assert(public_key_2.bytes().len() == 32);
    assert(public_key_2.bytes().get(31).unwrap() == 1u8);
    let mut iter_2 = 0;
    while iter_2 < public_key_2.bytes().len() - 1 {
        assert(public_key_2.bytes().get(iter_2).unwrap() == 0u8);
        iter_2 += 1;
    }

    let public_key_3 = PublicKey::from(b256::max());
    assert(public_key_3.bytes().len() == 32);
    let mut iter_3 = 0;
    while iter_3 < public_key_3.bytes().len() {
        assert(public_key_3.bytes().get(iter_3).unwrap() == 255u8);
        iter_3 += 1;
    }
}

#[test]
fn public_key_try_from_bytes() {
    let mut bytes_1 = Bytes::new();
    let public_key_1 = PublicKey::try_from(bytes_1);
    assert(public_key_1.is_none());

    let mut bytes_2 = Bytes::new();
    bytes_2.push(1u8);
    bytes_2.push(2u8);
    bytes_2.push(3u8);
    let public_key_2 = PublicKey::try_from(bytes_2);
    assert(public_key_2.is_none());

    let bytes_3 = Bytes::from(b256::zero());
    let public_key_3 = PublicKey::try_from(bytes_3).unwrap();
    assert(public_key_3.bytes().len() == 32);
    let mut iter_3 = 0;
    while iter_3 < public_key_3.bytes().len() {
        assert(public_key_3.bytes().get(iter_3).unwrap() == 0u8);
        iter_3 += 1;
    }

    let bytes_4 = Bytes::from(b256::max());
    let public_key_4 = PublicKey::try_from(bytes_4).unwrap();
    assert(public_key_4.bytes().len() == 32);
    let mut iter_4 = 0;
    while iter_4 < public_key_4.bytes().len() {
        assert(public_key_4.bytes().get(iter_4).unwrap() == 255u8);
        iter_4 += 1;
    }

    let bytes_5 = Bytes::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let public_key_5 = PublicKey::try_from(bytes_5).unwrap();
    assert(public_key_5.bytes().len() == 32);
    assert(public_key_5.bytes().get(31).unwrap() == 1u8);
    let mut iter_5 = 0;
    while iter_5 < public_key_5.bytes().len() - 1 {
        assert(public_key_5.bytes().get(iter_5).unwrap() == 0u8);
        iter_5 += 1;
    }

    let b256_tuple_6 = (b256::zero(), b256::zero());
    let bytes_6 = Bytes::from(raw_slice::from_parts::<u8>(__addr_of(b256_tuple_6), 64));
    let public_key_6 = PublicKey::try_from(bytes_6).unwrap();
    assert(public_key_6.bytes().len() == 64);
    let mut iter_6 = 0;
    while iter_6 < public_key_6.bytes().len() {
        assert(public_key_6.bytes().get(iter_6).unwrap() == 0u8);
        iter_6 += 1;
    }

    let b256_tuple_7 = (b256::max(), b256::max());
    let bytes_7 = Bytes::from(raw_slice::from_parts::<u8>(__addr_of(b256_tuple_7), 64));
    let public_key_7 = PublicKey::try_from(bytes_7).unwrap();
    assert(public_key_7.bytes().len() == 64);
    let mut iter_7 = 0;
    while iter_7 < public_key_7.bytes().len() {
        assert(public_key_7.bytes().get(iter_7).unwrap() == 255u8);
        iter_7 += 1;
    }

    let b256_tuple_8 = (
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    );
    let bytes_8 = Bytes::from(raw_slice::from_parts::<u8>(__addr_of(b256_tuple_8), 64));
    let public_key_8 = PublicKey::try_from(bytes_8).unwrap();
    assert(public_key_8.bytes().len() == 64);
    assert(public_key_8.bytes().get(63).unwrap() == 1u8);
    let mut iter_8 = 0;
    while iter_8 < public_key_8.bytes().len() - 1 {
        assert(public_key_8.bytes().get(iter_8).unwrap() == 0u8);
        iter_8 += 1;
    }
}

#[test]
fn public_key_try_into_b256_tuple() {
    let public_key_1 = PublicKey::from((b256::zero(), b256::zero()));
    let (result_1_1, result_2_1) = <PublicKey as TryInto<(b256, b256)>>::try_into(public_key_1).unwrap();
    assert(result_1_1 == b256::zero());
    assert(result_2_1 == b256::zero());

    let public_key_2 = PublicKey::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let (result_1_2, result_2_2) = <PublicKey as TryInto<(b256, b256)>>::try_into(public_key_2).unwrap();
    assert(result_1_2 == b256::zero());
    assert(
        result_2_2 == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );

    let public_key_3 = PublicKey::from((b256::max(), b256::max()));
    let (result_1_3, result_2_3) = <PublicKey as TryInto<(b256, b256)>>::try_into(public_key_3).unwrap();
    assert(result_1_3 == b256::max());
    assert(result_2_3 == b256::max());

    let public_key_4 = PublicKey::from(b256::zero());
    let result_4 = <PublicKey as TryInto<(b256, b256)>>::try_into(public_key_4);
    assert(result_4.is_none());
}

#[test]
fn public_key_try_into_b512() {
    let b512_1 = B512::from((b256::zero(), b256::zero()));
    let public_key_1 = PublicKey::from(b512_1);
    assert(<PublicKey as TryInto<B512>>::try_into(public_key_1).unwrap() == b512_1);

    let b512_2 = B512::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let public_key_2 = PublicKey::from(b512_2);
    assert(<PublicKey as TryInto<B512>>::try_into(public_key_2).unwrap() == b512_2);

    let b512_3 = B512::from((b256::max(), b256::max()));
    let public_key_3 = PublicKey::from(b512_3);
    assert(<PublicKey as TryInto<B512>>::try_into(public_key_3).unwrap() == b512_3);

    let public_key_4 = PublicKey::from(b256::zero());
    let result = <PublicKey as TryInto<B512>>::try_into(public_key_4);
    assert(result.is_none());
}

#[test]
fn public_key_try_into_b256() {
    let public_key_1 = PublicKey::from(b256::zero());
    let result_1: b256 = <PublicKey as TryInto<b256>>::try_into(public_key_1).unwrap();
    assert(result_1 == b256::zero());

    let public_key_2 = PublicKey::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let result_2: b256 = <PublicKey as TryInto<b256>>::try_into(public_key_2).unwrap();
    assert(result_2 == 0x0000000000000000000000000000000000000000000000000000000000000001);

    let public_key_3 = PublicKey::from(b256::max());
    let result_3: b256 = <PublicKey as TryInto<b256>>::try_into(public_key_3).unwrap();
    assert(result_3 == b256::max());

    let public_key_4 = PublicKey::from((b256::zero(), b256::zero()));
    let result_4 = <PublicKey as TryInto<b256>>::try_into(public_key_4);
    assert(result_4.is_none());
}

#[test]
fn public_key_eq() {
    let public_key_1 = PublicKey::from(b256::zero());
    let public_key_2 = PublicKey::from(b256::zero());
    let public_key_3 = PublicKey::from(b256::max());
    let public_key_4 = PublicKey::from(b256::max());
    let public_key_5 = PublicKey::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let public_key_6 = PublicKey::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let public_key_7 = PublicKey::from((b256::zero(), b256::zero()));
    let public_key_8 = PublicKey::from((b256::zero(), b256::zero()));
    let public_key_9 = PublicKey::from((b256::max(), b256::max()));
    let public_key_10 = PublicKey::from((b256::max(), b256::max()));
    let public_key_11 = PublicKey::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let public_key_12 = PublicKey::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));

    assert(public_key_1 == public_key_2);
    assert(public_key_3 == public_key_4);
    assert(public_key_5 == public_key_6);
    assert(public_key_7 == public_key_8);
    assert(public_key_9 == public_key_10);
    assert(public_key_11 == public_key_12);

    assert(public_key_1 != public_key_3);
    assert(public_key_1 != public_key_5);
    assert(public_key_1 != public_key_7);
    assert(public_key_1 != public_key_9);
    assert(public_key_1 != public_key_11);

    assert(public_key_3 != public_key_5);
    assert(public_key_3 != public_key_7);
    assert(public_key_3 != public_key_9);
    assert(public_key_3 != public_key_11);

    assert(public_key_5 != public_key_7);
    assert(public_key_5 != public_key_9);
    assert(public_key_5 != public_key_11);

    assert(public_key_7 != public_key_9);
    assert(public_key_7 != public_key_11);

    assert(public_key_9 != public_key_11);
}

#[test]
fn public_key_hash() {
    let zero_public_key = PublicKey::from(b256::zero());
    let result_1 = sha256(zero_public_key);
    assert(result_1 == 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);

    let one_public_key = PublicKey::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let result_2 = sha256(one_public_key);
    assert(result_2 == 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
}

#[test]
fn public_key_codec() {
    let public_key = PublicKey::new();
    log(public_key);
}
